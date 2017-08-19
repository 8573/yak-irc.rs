use Message;
use client::Result;
use connection;
use connection::Connection;
use connection::ConnectionPrivate;
use connection::GenericConnection;
use connection::GetPeerAddr;
use connection::ReceiveMessage;
use connection::SendMessage;
use mio;
#[cfg(feature = "pircolate")]
use pircolate;
use std::fmt;
use std::net::SocketAddr;
use string_cache::DefaultAtom as CachedString;

lazy_static! {
    static ref DEFAULT_REALNAME: CachedString = format!(
            "Connected with <{url}> v{ver}",
            url = env!("CARGO_PKG_HOMEPAGE"),
            ver = env!("CARGO_PKG_VERSION")
        ).into();
}

#[derive(Debug)]
pub struct Session {
    connection: GenericConnection,
    nickname: CachedString,
    username: CachedString,
    realname: CachedString,
}

#[derive(Copy, Clone, Debug)]
pub struct SessionBuilder<
    ConnField = Option<GenericConnection>,
    NicknameField = Option<CachedString>,
    UsernameField = Option<CachedString>,
    RealnameField = Option<CachedString>,
> where
    ConnField: Into<Option<GenericConnection>>,
    NicknameField: Into<Option<CachedString>>,
    UsernameField: Into<Option<CachedString>>,
    RealnameField: Into<Option<CachedString>>,
{
    connection: ConnField,
    nickname: NicknameField,
    username: UsernameField,
    realname: RealnameField,
}

impl<ConnField, NicknameField, UsernameField, RealnameField>
    SessionBuilder<ConnField, NicknameField, UsernameField, RealnameField>
where
    ConnField: Into<Option<GenericConnection>>,
    NicknameField: Into<Option<CachedString>>,
    UsernameField: Into<Option<CachedString>>,
    RealnameField: Into<Option<CachedString>>,
{
    pub fn connection<C>(
        self,
        value: C,
    ) -> SessionBuilder<GenericConnection, NicknameField, UsernameField, RealnameField>
    where
        C: Into<GenericConnection>,
    {
        let SessionBuilder {
            connection: _,
            nickname,
            username,
            realname,
        } = self;

        SessionBuilder {
            connection: value.into(),
            nickname,
            username,
            realname,
        }
    }

    pub fn nickname<S>(
        self,
        value: S,
    ) -> SessionBuilder<ConnField, CachedString, UsernameField, RealnameField>
    where
        S: Into<CachedString>,
    {
        let SessionBuilder {
            connection,
            nickname: _,
            username,
            realname,
        } = self;

        SessionBuilder {
            connection,
            nickname: value.into(),
            username,
            realname,
        }
    }

    pub fn username<S>(
        self,
        value: S,
    ) -> SessionBuilder<ConnField, NicknameField, CachedString, RealnameField>
    where
        S: Into<CachedString>,
    {
        let SessionBuilder {
            connection,
            nickname,
            username: _,
            realname,
        } = self;

        SessionBuilder {
            connection,
            nickname,
            username: value.into(),
            realname,
        }
    }

    pub fn realname<S>(
        self,
        value: S,
    ) -> SessionBuilder<ConnField, NicknameField, UsernameField, CachedString>
    where
        S: Into<CachedString>,
    {
        let SessionBuilder {
            connection,
            nickname,
            username,
            realname: _,
        } = self;

        SessionBuilder {
            connection,
            nickname,
            username,
            realname: value.into(),
        }
    }
}

pub fn build() -> SessionBuilder {
    SessionBuilder {
        connection: None,
        nickname: None,
        username: None,
        realname: None,
    }
}

impl<UsernameField, RealnameField>
    SessionBuilder<GenericConnection, CachedString, UsernameField, RealnameField>
where
    UsernameField: Into<Option<CachedString>>,
    RealnameField: Into<Option<CachedString>>,
    Self: fmt::Debug,
{
    pub fn start(self) -> Result<Session> {
        trace!(
            "[{}] Initiating session from {:?}",
            self.connection.peer_addr()?,
            self
        );

        let SessionBuilder {
            mut connection,
            nickname,
            username,
            realname,
        } = self;

        let username = username.into().unwrap_or(nickname.clone());
        let realname = realname.into().unwrap_or(DEFAULT_REALNAME.clone());

        connection.try_send(&pircolate::Message::try_from(
            format!("NICK {}", nickname),
        )?)?;
        connection.try_send(&pircolate::Message::try_from(
            format!("USER {} 8 * :{}", username, realname),
        )?)?;

        Ok(Session {
            connection,
            nickname,
            username,
            realname,
        })
    }
}

pub trait TryIntoSession {
    fn try_into_session(self) -> Result<Session>;
}

impl TryIntoSession for Session {
    fn try_into_session(self) -> Result<Session> {
        Ok(self)
    }
}

impl<UsernameField, RealnameField> TryIntoSession
    for SessionBuilder<GenericConnection, CachedString, UsernameField, RealnameField>
where
    UsernameField: Into<Option<CachedString>>,
    RealnameField: Into<Option<CachedString>>,
    Self: fmt::Debug,
{
    fn try_into_session(self) -> Result<Session> {
        self.start()
    }
}

impl ReceiveMessage for Session {
    fn recv<Msg>(&mut self) -> connection::Result<Option<Msg>>
    where
        Msg: Message,
    {
        self.connection.recv()
    }
}

impl SendMessage for Session {
    fn try_send<Msg>(&mut self, msg: &Msg) -> connection::Result<()>
    where
        Msg: Message,
    {
        self.connection.try_send(msg)
    }
}

impl GetPeerAddr for Session {
    fn peer_addr(&self) -> connection::Result<SocketAddr> {
        self.connection.peer_addr()
    }
}

impl ConnectionPrivate for Session {
    fn mio_registerable(&self) -> &mio::event::Evented {
        self.connection.mio_registerable()
    }

    fn mio_registration_interest(&self) -> mio::Ready {
        self.connection.mio_registration_interest()
    }

    fn mio_poll_opts(&self) -> mio::PollOpt {
        self.connection.mio_poll_opts()
    }
}
