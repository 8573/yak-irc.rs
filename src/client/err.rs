use super::SessionId;
use connection;
use message;
#[cfg(feature = "pircolate")]
use pircolate;
use std::borrow::Cow;
use std::io;
use util;

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    links {
        IrcUtil(util::irc::Error, util::irc::ErrorKind);
        Message(message::Error, message::ErrorKind);
        Connection(connection::Error, connection::ErrorKind);
        Pircolate(pircolate::error::Error, pircolate::error::ErrorKind)
            #[cfg(feature = "pircolate")];
    }

    errors {
        InternalLogicError(desc: Cow<'static, str>) {
            description(concat!("there is an error in the programming of `", module_path!(), "`"))
            display("There is an error in the programming of `{}`: {}", module_path!(), desc)
        }
        TooManySessions {
            description("an operation has failed because the client has too many sessions")
            display("An operation has failed because the client has too many sessions")
        }
        SessionIdFromWrongClient(session_id: SessionId, operation_name: Cow<'static, str>) {
            description("a client operation taking a `SessionId` was given a `SessionId` from the \
                         wrong client")
            display("A client operation ({:?}) taking a `SessionId` was given a `SessionId` from \
                     the wrong client: {:?}",
                    operation_name, session_id)
        }
    }
}
