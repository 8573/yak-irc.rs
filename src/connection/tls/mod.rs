use super::Connection;
use super::ConnectionPrivate;
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

mod inner;

#[derive(Debug)]
pub struct TlsConnection {
    inner: BufReader<inner::TlsClient>,
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
        let tls_client = inner::TlsClient::new(tcp_stream, hostname, config);

        trace!("[{}] Established TLS connection.", tcp_stream.peer_addr()?);

        let inner = BufReader::with_capacity(IRC_LINE_MAX_LEN, tls_client);

        Ok(TlsConnection { inner })
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

impl ConnectionPrivate for TlsConnection {
    fn mio_registerable(&self) -> &mio::event::Evented {
        &self.tcp_stream
    }

    fn mio_registration_interest(&self) -> mio::Ready {
        self.inner.ready_interest()
    }

    fn mio_poll_opts(&self) -> mio::PollOpt {
        mio::PollOpt::level() | mio::PollOpt::oneshot()
    }
}
