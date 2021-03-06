use super::Action;
use super::Client;
use super::ClientHandle;
use super::ClientPrivate;
use super::ErrorKind;
use super::MessageContext;
use super::Reaction;
use super::Result;
use super::ResultExt;
use super::SessionId;
use super::session::Session;
use super::session::TryIntoSession;
use Message;
use connection;
use connection::Connection;
use connection::ConnectionPrivate;
use connection::GenericConnection;
use connection::ReceiveMessage;
use connection::SendMessage;
use mio;
use smallvec::SmallVec;
use std;
use std::io;
use std::sync::mpsc;
use util;
use util::irc::pong_from_ping;
use uuid::Uuid;

mod tests;

const MPSC_QUEUE_SIZE_LIMIT: usize = 1024;

#[derive(Debug)]
pub struct ThinClient<Msg>
where
    Msg: Message,
{
    uuid: Uuid,
    sessions: SmallVec<[SessionEntry<Msg>; 3]>,
    mpsc_receiver: mpsc::Receiver<Action<Msg>>,
    mpsc_registration: mio::Registration,
    handle_prototype: ClientHandle<Msg>,
}

#[derive(Debug)]
pub(crate) struct SessionEntry<Msg>
where
    Msg: Message,
{
    inner: Session<GenericConnection>,
    output_queue: SmallVec<[Msg; 3]>,
    is_writable: bool,
}

/// Identifies the context associated with a `mio` event.
///
/// The context could be an IRC session, or it could be the MPSC queue via which the library
/// consumer can asynchronously send messages and other actions to this library.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
enum EventContextId {
    MpscQueue,
    Session(SessionId),
}

impl<Msg> ThinClient<Msg>
where
    Msg: Message,
{
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        let sessions = SmallVec::new();
        let (mpsc_sender, mpsc_receiver) = mpsc::sync_channel(MPSC_QUEUE_SIZE_LIMIT);
        let (mpsc_registration, readiness_setter) = mio::Registration::new2();
        let handle_prototype = ClientHandle {
            client_uuid: uuid,
            mpsc_sender,
            readiness_setter,
        };

        ThinClient {
            uuid,
            sessions,
            mpsc_receiver,
            mpsc_registration,
            handle_prototype,
        }
    }

    fn mk_event_ctx_id_from_mio_token(
        &self,
        mio::Token(token_number): mio::Token,
    ) -> EventContextId {
        match token_number {
            0 => EventContextId::MpscQueue,
            n => EventContextId::Session(SessionId {
                index: n - 1,
                client_uuid: self.uuid,
            }),
        }
    }
}

impl<Msg> ClientPrivate<Msg> for ThinClient<Msg>
where
    Msg: Message,
{
    fn mk_session_id(&self, session_index: usize) -> Result<SessionId> {
        let id = SessionId {
            index: session_index,
            client_uuid: self.uuid,
        };

        // Ensure that the session index can be converted to a Mio token.
        let mio::Token(_) = EventContextId::Session(id).as_mio_token()?;

        Ok(id)
    }
}

impl<Msg> Client<Msg> for ThinClient<Msg>
where
    Msg: Message,
{
    fn handle(&self) -> ClientHandle<Msg> {
        self.handle_prototype.clone()
    }

    fn add_session<Sess>(&mut self, session: Sess) -> Result<SessionId>
    where
        Sess: TryIntoSession<Conn>,
    {
        let index = self.sessions.len();

        // `usize::MAX` would mean that the upcoming `push` call would cause an overflow, assuming
        // the system had somehow not run out of memory.
        ensure!(index < std::usize::MAX, ErrorKind::TooManySessions);

        let id = self.mk_session_id(index)?;

        self.sessions.push(SessionEntry {
            inner: session.try_into_session()?,
            output_queue: SmallVec::new(),
            is_writable: false,
        });

        Ok(id)
    }

    fn run<MsgHandler>(mut self, msg_handler: MsgHandler) -> Result<()>
    where
        MsgHandler: Fn(&MessageContext<Msg>, Result<Msg>) -> Reaction<Msg>,
    {
        let poll = match mio::Poll::new() {
            Ok(p) => p,
            Err(err) => {
                error!("Failed to construct `mio::Poll`: {} ({:?})", err, err);
                bail!(err)
            }
        };

        let mut events = mio::Events::with_capacity(512);

        for (index, &SessionEntry { ref inner, .. }) in self.sessions.iter().enumerate() {
            poll.register(
                inner.mio_registerable(),
                EventContextId::Session(self.mk_session_id(index)?)
                    .as_mio_token()?,
                inner.mio_registration_interest(),
                inner.mio_poll_opts(),
            )?
        }

        poll.register(
            &self.mpsc_registration,
            EventContextId::MpscQueue.as_mio_token()?,
            mio::Ready::readable(),
            mio::PollOpt::edge(),
        )?;

        loop {
            let _event_qty = poll.poll(&mut events, None)?;

            for event in &events {
                match self.mk_event_ctx_id_from_mio_token(event.token()) {
                    EventContextId::MpscQueue => process_mpsc_queue(&mut self),
                    EventContextId::Session(session_id) => {
                        let &SessionEntry { ref mut inner, ref mut output_queue, ref mut is_writable } = self.sessions[session_id.index];
                        session.inner.process_mio_event(
                            event.readiness(),
                            session,
                            session_id,
                            &msg_handler,
                            &self.handle_prototype,
                        )
                    }
                }
            }
        }
    }
}

fn process_session_event<Msg, MsgHandler>(
    readiness: mio::Ready,
    session_id: SessionId,
    output
    msg_handler: &MsgHandler,
    client_handle: &ClientHandle<Msg>,
) where
    Msg: Message,
    MsgHandler: Fn(&MessageContext<Msg>, Result<Msg>) -> Reaction<Msg>,
{
    if readiness.is_writable() {
        session.is_writable = true;
    }

    if session.is_writable {
        process_writable(session, session_id, msg_handler, client_handle);
    }

    if readiness.is_readable() {
        process_readable(session, session_id, msg_handler, client_handle);
    }
}

fn process_readable<Msg, MsgHandler>(
    session: &mut SessionEntry<Msg>,
    session_id: SessionId,
    msg_handler: &MsgHandler,
    client_handle: &ClientHandle<Msg>,
) where
    Msg: Message,
    MsgHandler: Fn(&MessageContext<Msg>, Result<Msg>) -> Reaction<Msg>,
{
    let msg_ctx = MessageContext {
        client_handle: client_handle.clone(),
        session_id,
    };

    loop {
        let msg = match session.inner.recv::<Msg>() {
            Ok(Some(msg)) => Ok(msg),
            Ok(None) => break,
            Err(connection::Error(connection::ErrorKind::Io(ref err), _))
                if [io::ErrorKind::WouldBlock, io::ErrorKind::TimedOut].contains(&err.kind()) => {
                break
            }
            Err(err) => Err(err.into()),
        };

        let reaction = handle_message(msg_handler, &msg_ctx, msg);

        process_reaction(session, session_id, reaction);
    }
}

fn process_writable<Msg, MsgHandler>(
    session: &mut SessionEntry<Msg>,
    session_id: SessionId,
    msg_handler: &MsgHandler,
    client_handle: &ClientHandle<Msg>,
) where
    Msg: Message,
    MsgHandler: Fn(&MessageContext<Msg>, Result<Msg>) -> Reaction<Msg>,
{
    let mut msgs_consumed = 0;

    for msg in session.output_queue.iter() {
        match session.inner.try_send(msg) {
            Ok(()) => msgs_consumed += 1,
            Err(connection::Error(connection::ErrorKind::Io(ref err), _))
                if [io::ErrorKind::WouldBlock, io::ErrorKind::TimedOut].contains(&err.kind()) => {
                session.is_writable = false;
                break;
            }
            Err(err) => {
                msgs_consumed += 1;
                error!(
                    "[session {}] Failed to send message {:?} (error: {})",
                    session_id.index,
                    msg.to_str_lossy(),
                    err
                )
            }
        }
    }

    util::smallvec::discard_front(&mut session.output_queue, msgs_consumed)
        .chain_err(|| {
            ErrorKind::InternalLogicError(
                module_path!(),
                "Tried to discard more messages from an outgoing message queue than it contained."
                    .into(),
            )
        })
        .unwrap_or_else(|err| {
            let msg_ctx = MessageContext {
                client_handle: client_handle.clone(),
                session_id,
            };
            process_reaction(session, session_id, msg_handler(&msg_ctx, Err(err)))
        });
}

fn handle_message<Msg, MsgHandler>(
    msg_handler: &MsgHandler,
    msg_ctx: &MessageContext<Msg>,
    msg: Result<Msg>,
) -> Reaction<Msg>
where
    Msg: Message,
    MsgHandler: Fn(&MessageContext<Msg>, Result<Msg>) -> Reaction<Msg>,
{
    let msg = match msg {
        Ok(msg) => {
            if msg.command_bytes() == b"PING" {
                match pong_from_ping(msg) {
                    Ok(pong) => return Reaction::RawMsg(pong),
                    Err(err) => Err(err.into()),
                }
            } else {
                Ok(msg)
            }
        }
        Err(err) => Err(err),
    };

    msg_handler(&msg_ctx, msg)
}

fn process_reaction<Msg>(
    session: &mut SessionEntry<Msg>,
    session_id: SessionId,
    reaction: Reaction<Msg>,
) where
    Msg: Message,
{
    match reaction {
        Reaction::None => {}
        Reaction::RawMsg(ref msg) => session.send(session_id, msg),
        Reaction::Multi(reactions) => {
            for r in reactions {
                process_reaction(session, session_id, r);
            }
        }
    }
}

fn process_mpsc_queue<Msg>(client: &mut ThinClient<Msg>)
where
    Msg: Message,
{
    while let Ok(action) = client.mpsc_receiver.try_recv() {
        process_action(client, action)
    }
}

fn process_action<Msg>(client: &mut ThinClient<Msg>, action: Action<Msg>)
where
    Msg: Message,
{
    match action {
        Action::RawMsg {
            session_id,
            ref message,
        } => {
            let ref mut session = client.sessions[session_id.index];
            session.send(session_id, message)
        }
    }
}

impl<Msg> SessionEntry<Msg>
where
    Msg: Message,
{
    fn send(&mut self, session_id: SessionId, msg: &Msg) {
        match self.inner.try_send(msg) {
            Ok(()) => {
                // TODO: log the `session_id`.
            }
            Err(connection::Error(connection::ErrorKind::Io(ref err), _))
                if [io::ErrorKind::WouldBlock, io::ErrorKind::TimedOut].contains(&err.kind()) => {
                trace!(
                    "[session {}] Write would block or timed out; enqueueing message for later \
                     transmission: {:?}",
                    session_id.index,
                    msg.to_str_lossy()
                );
                self.is_writable = false;
                self.output_queue.push(msg.clone());
            }
            Err(err) => {
                error!(
                    "[session {}] Failed to send message {:?} (error: {})",
                    session_id.index,
                    msg.to_str_lossy(),
                    err
                )
            }
        }
    }
}

impl EventContextId {
    fn as_mio_token(&self) -> Result<mio::Token> {
        let token_number = match self {
            &EventContextId::MpscQueue => 0,
            &EventContextId::Session(SessionId {
                                         index,
                                         client_uuid: _,
                                     }) => {
                match index.checked_add(1) {
                    Some(std::usize::MAX) => {
                        // The conversion would result in `mio::Token(std::usize::MAX)`, which Mio
                        // uses as a special, reserved marker value.
                        bail!(ErrorKind::TooManySessions)
                    }
                    None => {
                        // The conversion would result in overflow in integer addition.
                        bail!(ErrorKind::TooManySessions)
                    }
                    Some(n) => n,
                }
            }
        };

        Ok(mio::Token(token_number))
    }
}
