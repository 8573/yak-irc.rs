use Message;
use message;
use std::borrow::Cow;

error_chain! {
    links {
        Message(message::Error, message::ErrorKind);
    }
}

// TODO: Write test cases.
pub fn pong_from_ping<Msg>(msg: Msg) -> Result<Msg>
where
    Msg: Message,
{
    let mut pong_bytes = msg.as_bytes().to_owned();

    // TODO: Skip over prefix and IRCv3 tags, if any, rather than assuming that the message starts
    // with the command, "PING". (<http://ircv3.net/specs/core/message-tags-3.2.html>)
    pong_bytes[1] = b'O';

    Ok(Msg::try_from(Cow::Owned(pong_bytes))?)
}
