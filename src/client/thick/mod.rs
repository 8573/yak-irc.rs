use super::ThinClient;
use Message;

#[derive(Debug)]
pub struct ThickClient<Msg>
where
    Msg: Message,
{
    thin: ThinClient<Msg>,
}
