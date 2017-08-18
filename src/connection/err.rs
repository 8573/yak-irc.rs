use message;
#[cfg(feature = "pircolate")]
use pircolate;
use std::io;
use std::str;

error_chain! {
    foreign_links {
        Io(io::Error);
        Utf8Error(str::Utf8Error);
    }

    links {
        Message(message::Error, message::ErrorKind);
        Pircolate(pircolate::error::Error, pircolate::error::ErrorKind)
            #[cfg(feature = "pircolate")];
    }

    errors {
        MessageTooLong(message: Vec<u8>) {
            description("an attempt was made to send an IRC message longer than supported by the \
                         IRC protocol")
            display("An attempt was made to send an IRC message longer than supported by the IRC \
                     protocol: {:?} (length: {:?})",
                    String::from_utf8_lossy(&message), message.len())
        }
    }
}
