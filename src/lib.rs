extern crate mio;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[cfg(feature = "pircolate")]
extern crate pircolate;

pub use self::message::Message;

pub mod connection;
pub mod client;
pub mod message;
