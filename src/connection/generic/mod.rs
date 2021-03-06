use super::Connection;
use super::ConnectionPrivate;
use super::GetPeerAddr;
use super::PlaintextConnection;
use super::ReceiveMessage;
use super::Result;
use super::SendMessage;
use super::TlsConnection;
use Message;
use mio;
use std::net::SocketAddr;

// TODO: add usage example.
/// A generic IRC connection.
///
/// This type can be constructed from any type implementing [`Connection`]. It uses an internal
/// `enum` type to represent [`Connection`] types from this crate without storing them on the heap
/// to create trait objects. [`Connection`] types from other crates, if supported at all, will most
/// likely be stored on the heap.
///
/// [`Connection`]: trait.Connection.html
#[derive(Debug)]
pub struct GenericConnection {
    inner: GenericConnectionInner,
}

#[derive(Debug)]
enum GenericConnectionInner {
    Tls(TlsConnection),
    Plaintext(PlaintextConnection),
}

macro_rules! impl_generic {
    ($($src:ty: $variant:ident;)*) => {
        $(impl From<$src> for GenericConnection {
            fn from(original: $src) -> Self {
                GenericConnection {
                    inner: GenericConnectionInner::$variant(original),
                }
            }
        })*

        impl SendMessage for GenericConnection {
            fn try_send<Msg>(&mut self, msg: &Msg) -> Result<()>
            where
                Msg: Message,
            {
                match self.inner {
                    $(GenericConnectionInner::$variant(ref mut conn) => conn.try_send(msg),)*
                }
            }
        }

        impl ReceiveMessage for GenericConnection {
            fn recv<Msg>(&mut self) -> Result<Option<Msg>>
            where
                Msg: Message,
            {
                match self.inner {
                    $(GenericConnectionInner::$variant(ref mut conn) => conn.recv(),)*
                }
            }
        }

        impl GetPeerAddr for GenericConnection {
            fn peer_addr(&self) -> Result<SocketAddr> {
                match self.inner {
                    $(GenericConnectionInner::$variant(ref conn) => conn.peer_addr(),)*
                }
            }
        }

        impl ConnectionPrivate for GenericConnection {
            fn mio_registerable(&self) -> &mio::event::Evented {
                match self.inner {
                    $(GenericConnectionInner::$variant(ref conn) => conn.mio_registerable(),)*
                }
            }

            fn mio_registration_interest(&self) -> mio::Ready {
                match self.inner {
                    $(GenericConnectionInner::$variant(ref conn) => {
                        conn.mio_registration_interest()
                    })*
                }
            }

            fn mio_poll_opts(&self) -> mio::PollOpt {
                match self.inner {
                    $(GenericConnectionInner::$variant(ref conn) => conn.mio_poll_opts(),)*
                }
            }
        }
    };
}

impl_generic!(
    TlsConnection: Tls;
    PlaintextConnection: Plaintext;
);

impl Connection for GenericConnection {}
