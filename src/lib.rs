extern crate mio;
extern crate smallvec;
extern crate string_cache;
extern crate uuid;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[cfg(feature = "pircolate")]
extern crate pircolate;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

pub use self::message::Message;

pub mod connection;
pub mod client;
pub mod message;

mod util;
