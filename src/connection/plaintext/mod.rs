use super::Connection;
use super::ErrorKind;
use super::GetMioTcpStream;
use super::GetPeerAddr;
use super::IRC_LINE_MAX_LEN;
use super::ReceiveMessage;
use super::Result;
use super::SendMessage;
use Message;
use mio;
use std::borrow::Cow;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::net::ToSocketAddrs;

/// TODO: Use pub_restricted once I get 1.18.
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
        let msg_bytes = msg.as_bytes();

        ensure!(
            msg_bytes.len() <= IRC_LINE_MAX_LEN,
            ErrorKind::MessageTooLong(msg_bytes.to_owned())
        );

        self.tcp_writer.write_all(msg_bytes)?;
        self.tcp_writer.write_all(b"\r\n")?;

        match self.tcp_writer.flush() {
            Ok(()) => debug!("Sent message: {:?}", msg.to_str_lossy()),
            Err(err) => {
                error!(
                    "Wrote but failed to flush message: {:?} (error: {})",
                    msg.to_str_lossy(),
                    err
                );
                bail!(err)
            }
        }

        Ok(())
    }
}

impl ReceiveMessage for PlaintextConnection {
    fn recv<Msg>(&mut self) -> Result<Option<Msg>>
    where
        Msg: Message,
    {
        let mut line = Vec::new();

        let bytes_read = self.tcp_reader.read_until(b'\n', &mut line)?;

        if bytes_read == 0 {
            return Ok(None);
        }

        while line.ends_with(b"\n") || line.ends_with(b"\r") {
            let _popped_char = line.pop();
        }

        debug!("Received message: {:?}", String::from_utf8_lossy(&line));

        Msg::try_from(Cow::Owned(line)).map(Some).map_err(
            Into::into,
        )
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
