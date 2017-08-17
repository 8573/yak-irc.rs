use super::ClientConfig;
use super::ThinClient;
use Message;
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug)]
pub struct ThickClient<Msg>
where
    Msg: Message,
{
    thin: ThinClient<Msg>,
    config: Arc<RwLock<ClientConfig<Msg>>>,
}
