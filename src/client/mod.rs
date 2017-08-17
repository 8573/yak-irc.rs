use self::action::Action;
pub use self::err::*;
pub use self::msg_ctx::MessageContext;
pub use self::reaction::Reaction;
pub use self::thick::Client;
pub use self::thin::ThinClient;
use Message;
use mio;
use std::sync::mpsc;

pub mod session;

mod action;
mod err;
mod msg_ctx;
mod reaction;
mod thick;
mod thin;

#[derive(Clone, Debug)]
pub struct ClientHandle<Msg>
where
    Msg: Message,
{
    mpsc_sender: mpsc::SyncSender<Action<Msg>>,
    readiness_setter: mio::SetReadiness,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SessionId {
    index: usize,
}

impl<Msg> ClientHandle<Msg>
where
    Msg: Message,
{
    pub fn try_send(&mut self, session_id: SessionId, message: Msg) -> Result<()> {
        // Add the action to the client's MPSC queue.
        self.mpsc_sender
            .try_send(Action::RawMsg {
                session_id,
                message,
            })
            .unwrap();

        self.set_ready()?;

        Ok(())
    }

    /// Notifies the associated client that there's an action to read from the MPSC queue.
    fn set_ready(&self) -> Result<()> {
        self.readiness_setter.set_readiness(mio::Ready::readable())?;

        Ok(())
    }
}
