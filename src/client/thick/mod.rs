use super::ThinClient;
use Message;

#[derive(Debug)]
pub struct Client<Msg>
where
    Msg: Message,
{
    thin: ThinClient<Msg>,
}
