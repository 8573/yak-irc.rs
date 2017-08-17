use super::ClientHandle;
use super::SessionId;
use Message;

#[derive(Debug)]
pub struct MessageContext<Msg>
where
    Msg: Message,
{
    // TODO: Make these fields `pub_restricted` once I get 1.18.
    pub client_handle: ClientHandle<Msg>,
    pub session_id: SessionId,
}

impl<Msg> MessageContext<Msg>
where
    Msg: Message,
{
    pub fn client_handle(&self) -> &ClientHandle<Msg> {
        &self.client_handle
    }

    pub fn session_id(&self) -> SessionId {
        self.session_id
    }
}
