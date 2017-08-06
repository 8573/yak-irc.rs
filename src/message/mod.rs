pub use self::err::*;
#[cfg(feature = "pircolate")]
use pircolate;
use std::borrow::Cow;
use std::fmt;
use std::str;

mod err;

pub trait Message: Clone + fmt::Debug {
    fn try_from<'a>(Cow<'a, [u8]>) -> Result<Self>
    where
        Self: Sized;

    fn as_bytes(&self) -> &[u8];

    fn as_str(&self) -> Option<&str> {
        None
    }

    fn to_str(&self) -> Result<&str> {
        Ok(match self.as_str() {
            Some(s) => s,
            None => str::from_utf8(self.as_bytes())?,
        })
    }

    fn to_str_lossy<'a>(&'a self) -> Cow<'a, str> {
        match self.as_str() {
            Some(s) => Cow::Borrowed(s),
            None => String::from_utf8_lossy(self.as_bytes()),
        }
    }

    fn command_bytes(&self) -> &[u8];
}

#[cfg(feature = "pircolate")]
impl Message for pircolate::Message {
    fn try_from<'a>(input: Cow<'a, [u8]>) -> Result<Self> {
        Ok(Self::try_from(String::from_utf8(input.into_owned())?)?)
    }

    fn as_bytes(&self) -> &[u8] {
        self.raw_message().as_bytes()
    }

    fn as_str(&self) -> Option<&str> {
        Some(self.raw_message())
    }

    fn command_bytes(&self) -> &[u8] {
        self.raw_command().as_bytes()
    }
}
