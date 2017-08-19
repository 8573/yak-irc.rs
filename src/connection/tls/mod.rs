use super::Connection;
use super::GetMioTcpStream;
use super::GetPeerAddr;
use super::IRC_LINE_MAX_LEN;
use super::ReceiveMessage;
use super::Result;
use super::SendMessage;
use super::recv_common;
use super::try_send_common;
use Message;
use mio;
use rustls;
use rustls::Session as RustlsSession;
use std::io::BufReader;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::sync::Arc;

#[derive(Debug)]
pub struct TlsConnection {
    tcp_stream: mio::net::TcpStream,
    tls_session: BufReader<rustls::ClientSession>,
}

impl TlsConnection {
    pub fn from_addr<A>(
        server_addrs: A,
        config: &Arc<rustls::ClientConfig>,
        hostname: &str,
    ) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        Self::from_tcp_stream(TcpStream::connect(server_addrs)?, config, hostname)
    }

    pub fn from_tcp_stream(
        tcp_stream: TcpStream,
        config: &Arc<rustls::ClientConfig>,
        hostname: &str,
    ) -> Result<Self> {
        let tcp_stream = mio::net::TcpStream::from_stream(tcp_stream)?;
        let tls_session = rustls::ClientSession::new(config, hostname);
        let tls_session = BufReader::with_capacity(IRC_LINE_MAX_LEN, tls_session);

        trace!("[{}] Established TLS connection.", tcp_stream.peer_addr()?);

        Ok(TlsConnection {
            tcp_stream,
            tls_session,
        })
    }

    fn complete_io(&mut self) -> Result<()> {
        let (_bytes_read, _bytes_written) =
            self.tls_session.get_mut().complete_io(&mut self.tcp_stream)?;

        Ok(())
    }

    /// An internal method.
    ///
    /// This method is derived from, and the trait implementations that use it are based on, the
    /// implementation of `rustls::Session` in version 0.10.0 of `rustls`, which comes with the
    /// following notices:
    ///
    /// > Copyright (c) 2016, Joseph Birr-Pixton <jpixton@gmail.com>
    /// >
    /// > Permission to use, copy, modify, and/or distribute this software for
    /// > any purpose with or without fee is hereby granted, provided that the
    /// > above copyright notice and this permission notice appear in all copies.
    fn complete_prior_io(&mut self) -> Result<()> {
        if self.tls_session.get_ref().is_handshaking() {
            self.complete_io()?;
        }

        if self.tls_session.get_ref().wants_write() {
            self.complete_io()?;
        }

        Ok(())
    }
}

impl Connection for TlsConnection {}

impl ReceiveMessage for TlsConnection {
    fn recv<Msg>(&mut self) -> Result<Option<Msg>>
    where
        Msg: Message,
    {
        self.complete_prior_io()?;

        if self.tls_session.get_ref().wants_read() {
            self.complete_io()?;
        }

        let msg = recv_common(&mut self.tls_session)?;

        Ok(msg)
    }
}

impl SendMessage for TlsConnection {
    fn try_send<Msg>(&mut self, msg: &Msg) -> Result<()>
    where
        Msg: Message,
    {
        self.complete_prior_io()?;

        try_send_common(self.tls_session.get_mut(), msg)?;

        self.complete_io()?;

        Ok(())
    }
}

impl GetPeerAddr for TlsConnection {
    fn peer_addr(&self) -> Result<SocketAddr> {
        Ok(self.tcp_stream.peer_addr()?)
    }
}

impl GetMioTcpStream for TlsConnection {
    fn mio_tcp_stream(&self) -> &mio::net::TcpStream {
        &self.tcp_stream
    }
}
