use Message;
use std::fmt;

pub struct ClientConfig<Msg>
where
    Msg: Message,
{
    // TODO: Delete this field once real fields are added.
    _msg: Msg,
}

impl<Msg> fmt::Debug for ClientConfig<Msg>
where
    Msg: Message,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &ClientConfig { _msg: _ } = self;

        f.debug_struct(stringify!(ClientConfig))
            //.field(stringify!(... name ...), ... value ...)
            .finish()
    }
}
