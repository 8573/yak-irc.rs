pub use self::err::*;
pub use self::generic::GenericConnection;
pub use self::plaintext::PlaintextConnection;
use Message;
use mio;
use std::borrow::Cow;
use std::fmt::Debug;
use std::io::BufRead;
use std::io::Write;
use std::net::SocketAddr;

mod err;
mod generic;
mod plaintext;

const IRC_LINE_MAX_LEN: usize = 1024;

pub trait Connection
    : Send + ReceiveMessage + SendMessage + GetPeerAddr + Into<GenericConnection> + Debug
    {
}

pub trait SendMessage: Send + GetPeerAddr + Debug {
    fn try_send<Msg>(&mut self, &Msg) -> Result<()>
    where
        Msg: Message;
}

pub trait ReceiveMessage: Send + GetPeerAddr + Debug {
    /// Must perform a blocking read. Must return `Ok(None)` if there is no message to return, and
    /// not otherwise.
    fn recv<Msg>(&mut self) -> Result<Option<Msg>>
    where
        Msg: Message;
}

pub trait GetPeerAddr {
    fn peer_addr(&self) -> Result<SocketAddr>;
}

/// TODO: Use pub_restricted once I get 1.18.
pub trait GetMioTcpStream {
    /// Returns a reference to `self`'s underlying `mio::net::TcpStream`, which is intended solely
    /// for registering the `TcpStream` with a `mio::Poll`.
    fn mio_tcp_stream(&self) -> &mio::net::TcpStream;
}

fn recv_common<R, Msg>(reader: &mut R) -> Result<Option<Msg>>
where
    R: BufRead,
    Msg: Message,
{
    let mut line = Vec::new();

    let bytes_read = reader.read_until(b'\n', &mut line)?;

    if bytes_read == 0 {
        return Ok(None);
    }

    while line.ends_with(b"\n") || line.ends_with(b"\r") {
        let _popped_char = line.pop();
    }

    debug!("Received message: {:?}", String::from_utf8_lossy(&line));

    Ok(Msg::try_from(Cow::Owned(line)).map(Some)?)
}

fn try_send_common<W, Msg>(writer: &mut W, msg: &Msg) -> Result<()>
where
    W: Write,
    Msg: Message,
{
    let msg_bytes = msg.as_bytes();

    ensure!(
        msg_bytes.len() <= IRC_LINE_MAX_LEN,
        ErrorKind::MessageTooLong(msg_bytes.to_owned())
    );

    writer.write_all(msg_bytes)?;
    writer.write_all(b"\r\n")?;

    match writer.flush() {
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
