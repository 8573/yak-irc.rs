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
use std::fmt;
use std::io::BufReader;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Arc;

pub struct TlsConnection {
    tcp_stream: mio::net::TcpStream,
    tls_session: BufReader<rustls::ClientSession>,
}

impl TlsConnection {
    pub fn new(config: &Arc<rustls::ClientConfig>, hostname: &str) -> Result<Self> {
        let tcp_stream = TcpStream::connect(hostname)?;
        let tcp_stream = mio::net::TcpStream::from_stream(tcp_stream)?;
        let tls_session = rustls::ClientSession::new(config, hostname);
        let tls_session = BufReader::with_capacity(IRC_LINE_MAX_LEN, tls_session);

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
}

impl Connection for TlsConnection {}

impl ReceiveMessage for TlsConnection {
    fn recv<Msg>(&mut self) -> Result<Option<Msg>>
    where
        Msg: Message,
    {
        let msg = recv_common(&mut self.tls_session)?;
        self.complete_io()?;

        Ok(msg)
    }
}

impl SendMessage for TlsConnection {
    fn try_send<Msg>(&mut self, msg: &Msg) -> Result<()>
    where
        Msg: Message,
    {
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

// TODO: Once I get rustc 1.18, update to rustls 0.10, derive `Debug` for `TlsConnection`, and
// delete this stop-gap `impl`.
impl fmt::Debug for TlsConnection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(stringify!(TlsConnection)).finish()
    }
}
