use Message;
use client::Result;
use connection;
use connection::Connection;
use connection::GenericConnection;
use connection::GetMioTcpStream;
use connection::GetPeerAddr;
use connection::ReceiveMessage;
use connection::SendMessage;
use mio;
#[cfg(feature = "pircolate")]
use pircolate;
use std::fmt;
use std::marker::PhantomData;
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
pub struct Session<Conn>
where
    Conn: Connection,
{
    connection: Conn,
    nickname: CachedString,
    username: CachedString,
    realname: CachedString,
}

#[derive(Copy, Clone, Debug)]
pub struct SessionBuilder<
    Conn,
    ConnField = Option<Conn>,
    NicknameField = Option<CachedString>,
    UsernameField = Option<CachedString>,
    RealnameField = Option<CachedString>,
> where
    Conn: Connection,
    ConnField: Into<Option<Conn>>,
    NicknameField: Into<Option<CachedString>>,
    UsernameField: Into<Option<CachedString>>,
    RealnameField: Into<Option<CachedString>>,
{
    connection: ConnField,
    nickname: NicknameField,
    username: UsernameField,
    realname: RealnameField,
    _result_phantom: PhantomData<Session<Conn>>,
}

impl<Conn, ConnField, NicknameField, UsernameField, RealnameField>
    SessionBuilder<Conn, ConnField, NicknameField, UsernameField, RealnameField>
where
    Conn: Connection,
    ConnField: Into<Option<Conn>>,
    NicknameField: Into<Option<CachedString>>,
    UsernameField: Into<Option<CachedString>>,
    RealnameField: Into<Option<CachedString>>,
{
    pub fn connection(
        self,
        value: Conn,
    ) -> SessionBuilder<Conn, Conn, NicknameField, UsernameField, RealnameField> {
        let SessionBuilder {
            connection: _,
            nickname,
            username,
            realname,
            _result_phantom,
        } = self;

        SessionBuilder {
            connection: value,
            nickname,
            username,
            realname,
            _result_phantom,
        }
    }

    pub fn nickname<S>(
        self,
        value: S,
    ) -> SessionBuilder<Conn, ConnField, CachedString, UsernameField, RealnameField>
    where
        S: Into<CachedString>,
    {
        let SessionBuilder {
            connection,
            nickname: _,
            username,
            realname,
            _result_phantom,
        } = self;

        SessionBuilder {
            connection,
            nickname: value.into(),
            username,
            realname,
            _result_phantom,
        }
    }

    pub fn username<S>(
        self,
        value: S,
    ) -> SessionBuilder<Conn, ConnField, NicknameField, CachedString, RealnameField>
    where
        S: Into<CachedString>,
    {
        let SessionBuilder {
            connection,
            nickname,
            username: _,
            realname,
            _result_phantom,
        } = self;

        SessionBuilder {
            connection,
            nickname,
            username: value.into(),
            realname,
            _result_phantom,
        }
    }

    pub fn realname<S>(
        self,
        value: S,
    ) -> SessionBuilder<Conn, ConnField, NicknameField, UsernameField, CachedString>
    where
        S: Into<CachedString>,
    {
        let SessionBuilder {
            connection,
            nickname,
            username,
            realname: _,
            _result_phantom,
        } = self;

        SessionBuilder {
            connection,
            nickname,
            username,
            realname: value.into(),
            _result_phantom,
        }
    }
}

pub fn build<Conn>() -> SessionBuilder<Conn>
where
    Conn: Connection,
{
    SessionBuilder {
        connection: None,
        nickname: None,
        username: None,
        realname: None,
        _result_phantom: Default::default(),
    }
}

impl<Conn, UsernameField, RealnameField>
    SessionBuilder<Conn, Conn, CachedString, UsernameField, RealnameField>
where
    Conn: Connection,
    UsernameField: Into<Option<CachedString>>,
    RealnameField: Into<Option<CachedString>>,
    Self: fmt::Debug,
{
    pub fn start(self) -> Result<Session<Conn>> {
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
            _result_phantom: _,
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

pub trait TryIntoSession<Conn>
where
    Conn: Connection,
{
    fn try_into_session(self) -> Result<Session<Conn>>;
}

impl<Conn> TryIntoSession<Conn> for Session<Conn>
where
    Conn: Connection,
{
    fn try_into_session(self) -> Result<Session<Conn>> {
        Ok(self)
    }
}

impl<Conn, UsernameField, RealnameField> TryIntoSession<Conn>
    for SessionBuilder<Conn, Conn, CachedString, UsernameField, RealnameField>
where
    Conn: Connection,
    UsernameField: Into<Option<CachedString>>,
    RealnameField: Into<Option<CachedString>>,
    Self: fmt::Debug,
{
    fn try_into_session(self) -> Result<Session<Conn>> {
        self.start()
    }
}

impl<Conn> Session<Conn>
where
    Conn: Connection,
{
    pub fn into_generic(self) -> Session<GenericConnection> {
        let Session {
            connection,
            nickname,
            username,
            realname,
        } = self;

        Session {
            connection: connection.into(),
            nickname,
            username,
            realname,
        }
    }
}

impl<Conn> ReceiveMessage for Session<Conn>
where
    Conn: Connection,
{
    fn recv<Msg>(&mut self) -> connection::Result<Option<Msg>>
    where
        Msg: Message,
    {
        self.connection.recv()
    }
}

impl<Conn> SendMessage for Session<Conn>
where
    Conn: Connection,
{
    fn try_send<Msg>(&mut self, msg: &Msg) -> connection::Result<()>
    where
        Msg: Message,
    {
        self.connection.try_send(msg)
    }
}

impl<Conn> GetPeerAddr for Session<Conn>
where
    Conn: Connection,
{
    fn peer_addr(&self) -> connection::Result<SocketAddr> {
        self.connection.peer_addr()
    }
}

impl<Conn> GetMioTcpStream for Session<Conn>
where
    Conn: Connection + GetMioTcpStream,
{
    fn mio_tcp_stream(&self) -> &mio::net::TcpStream {
        self.connection.mio_tcp_stream()
    }
}
