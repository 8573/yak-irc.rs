use connection;
use message;
#[cfg(feature = "pircolate")]
use pircolate;
use std::io;

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    links {
        Message(message::Error, message::ErrorKind);
        Connection(connection::Error, connection::ErrorKind);
        Pircolate(pircolate::error::Error, pircolate::error::ErrorKind)
            #[cfg(feature = "pircolate")];
    }

    errors {
        TooManySessions {
            description("an operation has failed because the client has too many sessions")
            display("An operation has failed because the client has too many sessions")
        }
    }
}
