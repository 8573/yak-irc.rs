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
use std::io::BufReader;
use std::io::BufWriter;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::net::ToSocketAddrs;

#[derive(Debug)]
pub struct PlaintextConnection {
    tcp_reader: BufReader<mio::net::TcpStream>,
    tcp_writer: BufWriter<mio::net::TcpStream>,
}

impl PlaintextConnection {
    pub fn from_addr<A>(server_addrs: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        Self::from_tcp_stream(TcpStream::connect(server_addrs)?)
    }

    pub fn from_tcp_stream(tcp_reader: TcpStream) -> Result<Self> {
        let tcp_reader = mio::net::TcpStream::from_stream(tcp_reader)?;

        trace!(
            "[{}] Established plaintext connection.",
            tcp_reader.peer_addr()?
        );

        let tcp_writer = BufWriter::with_capacity(IRC_LINE_MAX_LEN, tcp_reader.try_clone()?);
        let tcp_reader = BufReader::with_capacity(IRC_LINE_MAX_LEN, tcp_reader);

        Ok(PlaintextConnection {
            tcp_reader,
            tcp_writer,
        })
    }
}

impl Connection for PlaintextConnection {}

impl SendMessage for PlaintextConnection {
    fn try_send<Msg>(&mut self, msg: &Msg) -> Result<()>
    where
        Msg: Message,
    {
        try_send_common(&mut self.tcp_writer, msg)
    }
}

impl ReceiveMessage for PlaintextConnection {
    fn recv<Msg>(&mut self) -> Result<Option<Msg>>
    where
        Msg: Message,
    {
        recv_common(&mut self.tcp_reader)
    }
}

impl GetPeerAddr for PlaintextConnection {
    fn peer_addr(&self) -> Result<SocketAddr> {
        self.tcp_reader.get_ref().peer_addr().map_err(Into::into)
    }
}

impl GetMioTcpStream for PlaintextConnection {
    fn mio_tcp_stream(&self) -> &mio::net::TcpStream {
        self.tcp_reader.get_ref()
    }
}
