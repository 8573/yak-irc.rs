use super::SessionId;
use Message;

pub enum Action<Msg>
where
    Msg: Message,
{
    /// Send a message like `Reaction::RawMsg`, in a specified session.
    RawMsg { session_id: SessionId, message: Msg },

    // TODO: Add a `Quit` action.
}
