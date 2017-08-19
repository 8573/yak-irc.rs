//! Maybe this makes rustls work with mio.
//!
//! This code was derived from the file `examples/tlsclient.rs` in version 0.10.0 of `rustls`,
//! which comes with the following notices:
//!
//! > Copyright (c) 2016, Joseph Birr-Pixton <jpixton@gmail.com>
//! >
//! > Permission to use, copy, modify, and/or distribute this software for
//! > any purpose with or without fee is hereby granted, provided that the
//! > above copyright notice and this permission notice appear in all copies.

use mio;
use mio::tcp::TcpStream;
use rustls;
use rustls::Session;
use std::collections;
use std::fs;
use std::io;
use std::io::{BufReader, Read, Write};
use std::net::SocketAddr;
use std::process;
use std::str;
use std::sync::Arc;
use std::sync::Mutex;

/// This encapsulates the TCP-level connection, some connection
/// state, and the underlying TLS-level session.
pub struct TlsClient {
    socket: TcpStream,
    closing: bool,
    clean_closure: bool,
    tls_session: rustls::ClientSession,
}

impl TlsClient {
    fn ready(&mut self, poll: &mut mio::Poll, ev: &mio::Event) {
        if ev.readiness().is_readable() {
            self.do_read();
        }

        if ev.readiness().is_writable() {
            self.do_write();
        }

        if self.is_closed() {
            println!("Connection closed");
            process::exit(if self.clean_closure { 0 } else { 1 });
        }

        self.reregister(poll);
    }
}

/// We implement `io::Write` and pass through to the TLS session
impl io::Write for TlsClient {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.tls_session.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tls_session.flush()
    }
}

impl io::Read for TlsClient {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.tls_session.read(bytes)
    }
}

impl TlsClient {
    fn new(sock: TcpStream, hostname: &str, cfg: Arc<rustls::ClientConfig>) -> TlsClient {
        TlsClient {
            socket: sock,
            closing: false,
            clean_closure: false,
            tls_session: rustls::ClientSession::new(&cfg, hostname),
        }
    }

    fn read_source_to_end(&mut self, rd: &mut io::Read) -> io::Result<usize> {
        let mut buf = Vec::new();
        let len = rd.read_to_end(&mut buf)?;
        self.tls_session.write_all(&buf).unwrap();
        Ok(len)
    }

    /// We're ready to do a read.
    fn do_read(&mut self) {
        // Read TLS data.  This fails if the underlying TCP connection
        // is broken.
        let rc = self.tls_session.read_tls(&mut self.socket);
        if rc.is_err() {
            println!("TLS read error: {:?}", rc);
            self.closing = true;
            return;
        }

        // If we're ready but there's no data: EOF.
        if rc.unwrap() == 0 {
            println!("EOF");
            self.closing = true;
            self.clean_closure = true;
            return;
        }

        // Reading some TLS data might have yielded new TLS
        // messages to process.  Errors from this indicate
        // TLS protocol problems and are fatal.
        let processed = self.tls_session.process_new_packets();
        if processed.is_err() {
            println!("TLS error: {:?}", processed.unwrap_err());
            self.closing = true;
            return;
        }

        // Having read some TLS data, and processed any new messages,
        // we might have new plaintext as a result.
        //
        // Read it and then write it to stdout.
        let mut plaintext = Vec::new();
        let rc = self.tls_session.read_to_end(&mut plaintext);
        if !plaintext.is_empty() {
            io::stdout().write_all(&plaintext).unwrap();
        }

        // If that fails, the peer might have started a clean TLS-level
        // session closure.
        if rc.is_err() {
            let err = rc.unwrap_err();
            println!("Plaintext read error: {:?}", err);
            self.clean_closure = err.kind() == io::ErrorKind::ConnectionAborted;
            self.closing = true;
            return;
        }
    }

    fn do_write(&mut self) {
        self.tls_session.write_tls(&mut self.socket).unwrap();
    }

    fn register(&self, poll: &mut mio::Poll) {
        poll.register(
            &self.socket,
            CLIENT,
            self.ready_interest(),
            mio::PollOpt::level() | mio::PollOpt::oneshot(),
        ).unwrap();
    }

    fn reregister(&self, poll: &mut mio::Poll) {
        poll.reregister(
            &self.socket,
            CLIENT,
            self.ready_interest(),
            mio::PollOpt::level() | mio::PollOpt::oneshot(),
        ).unwrap();
    }

    // Use wants_read/wants_write to register for different mio-level
    // IO readiness events.
    fn ready_interest(&self) -> mio::Ready {
        let rd = self.tls_session.wants_read();
        let wr = self.tls_session.wants_write();

        if rd && wr {
            mio::Ready::readable() | mio::Ready::writable()
        } else if wr {
            mio::Ready::writable()
        } else {
            mio::Ready::readable()
        }
    }

    fn is_closed(&self) -> bool {
        self.closing
    }
}

/// Parse some arguments, then make a TLS client connection
/// somewhere.
fn main() {
    let sock = TcpStream::connect(&addr).unwrap();
    let mut tlsclient = TlsClient::new(sock, &args.arg_hostname, config);

    if args.flag_http {
        let httpreq = format!(
            "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept-Encoding: \
             identity\r\n\r\n",
            args.arg_hostname
        );
        tlsclient.write_all(httpreq.as_bytes()).unwrap();
    } else {
        let mut stdin = io::stdin();
        tlsclient.read_source_to_end(&mut stdin).unwrap();
    }

    let mut poll = mio::Poll::new().unwrap();
    let mut events = mio::Events::with_capacity(32);
    tlsclient.register(&mut poll);

    loop {
        poll.poll(&mut events, None).unwrap();

        for ev in events.iter() {
            tlsclient.ready(&mut poll, &ev);
        }
    }
}
