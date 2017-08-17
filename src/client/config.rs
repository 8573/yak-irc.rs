use super::MessageContext;
use super::Reaction;
use super::Result;
use Message;
use std::fmt;
use util;

pub struct ClientConfig<Msg>
where
    Msg: Message,
{
    pub msg_handler: Box<Fn(&MessageContext<Msg>, Result<Msg>) -> Reaction<Msg>>,
}

impl<Msg> fmt::Debug for ClientConfig<Msg>
where
    Msg: Message,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &ClientConfig { msg_handler: _ } = self;

        f.debug_struct(stringify!(ClientConfig))
            .field(stringify!(msg_handler), &util::fmt::debug_repr::Fn)
            .finish()
    }
}
