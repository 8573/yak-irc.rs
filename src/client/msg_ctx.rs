use super::SessionId;

#[derive(Debug)]
pub struct MessageContext {
    // TODO: Include a `ClientHandle` or similar.
    // TODO: Make these fields `pub_restricted` once I get 1.18.
    pub session_id: SessionId,
}

impl MessageContext {
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }
}
