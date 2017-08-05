pub use self::err::*;
pub use self::generic::GenericConnection;
pub use self::plaintext::PlaintextConnection;
use Message;
use mio;
use std::fmt::Debug;
use std::net::SocketAddr;

mod err;
mod generic;
mod plaintext;

#[cfg(auto_send_recv_threads)]
mod auto_threading;

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
