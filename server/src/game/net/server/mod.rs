/*! TCP server implementation for managing connected game clients */

mod connection_manager;
pub mod packet;

use mio::net::TcpListener;
use mio::{Events, Interest, Poll};

use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::sync::mpsc;
use std::time::Duration;

use bevy::log;

#[derive(Debug, Default)]
/// Information about each active client to be sent to the client.
pub struct ActiveClientInfo(pub HashMap<Token, lib_spells::net::ClientInfo>);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token(mio::Token);

impl Token {
    pub fn new(id: usize) -> Self {
        Self(mio::Token(id))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client ID {}", self.0 .0)
    }
}

impl From<mio::Token> for Token {
    fn from(value: mio::Token) -> Self {
        Self(value)
    }
}

impl From<Token> for mio::Token {
    fn from(value: Token) -> Self {
        value.0
    }
}

const SERVER_TOKEN: Token = Token(mio::Token(0));

const EVENT_BUFFER_SIZE: usize = 1028;
const MIN_TICK: Duration = Duration::from_millis(100);

// these should be in a passed in config
const SERVER_ADDR: &str = "0.0.0.0:7776";
// ^

#[derive(Debug)]
pub enum Incoming {
    Joined(Token),
    Left(Token),
    Data(Token, packet::Packet),
}

#[derive(Debug)]
pub enum Outgoing {
    Kick(Token),
    Broadcast(Vec<u8>),
    ClientInfo(ActiveClientInfo),
}

pub struct Server {
    listener: TcpListener,
    events: Events,
    poll: Poll,
}

impl Server {
    pub fn create() -> io::Result<Server> {
        log::info!("binding server to {SERVER_ADDR}");
        let mut listener = TcpListener::bind(SERVER_ADDR.parse().unwrap())?;
        let poll = Poll::new()?;
        poll.registry()
            .register(&mut listener, SERVER_TOKEN.into(), Interest::READABLE)?;
        let events = Events::with_capacity(EVENT_BUFFER_SIZE);
        Ok(Server {
            listener,
            poll,
            events,
        })
    }

    /// block on event look waiting for new clients, adding them by their token to a map of active cleint
    pub fn event_loop(
        &mut self,
        inc_tx: mpsc::Sender<Incoming>,
        out_rx: mpsc::Receiver<Outgoing>,
        password: Option<String>,
    ) -> io::Result<()> {
        let mut manager = connection_manager::ConnectionManager::new(inc_tx, out_rx, password);

        let mut next_socket = 1_usize;
        let mut next_token = || {
            let token = Token::new(next_socket);
            next_socket += 1;
            token
        };

        loop {
            self.poll
                .poll(&mut self.events, Some(MIN_TICK))
                .expect("poll died");

            manager.tick();
            manager.collect_dead(|dead| {
                log::debug!("deregister dead: {}", dead);
                self.poll
                    .registry()
                    .deregister(&mut dead.into_inner())
                    .expect("poll dead");
            });
            for ev in self.events.iter() {
                match ev.token().into() {
                    // new connections inc
                    SERVER_TOKEN => loop {
                        let (mut stream, addr) = match self.listener.accept() {
                            Ok((stream, addr)) => (stream, addr),
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(e) => return Err(e),
                        };
                        let new_token = next_token();
                        log::info!("got new connection from: {}, assigned: {}", addr, new_token);
                        self.poll
                            .registry()
                            .register(
                                &mut stream,
                                new_token.into(),
                                Interest::READABLE.add(Interest::WRITABLE),
                            )
                            .unwrap();
                        manager.manage_stream(
                            new_token,
                            connection_manager::tcp_stream::ClientStream::new(stream),
                            ev.is_readable(),
                        );
                    },
                    // this is a managed client
                    token => {
                        if ev.is_readable() {
                            manager.try_read(token);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Read, io::Write, sync::mpsc, thread};
    use super::*;

    #[ignore]
    #[test]
    fn test_incoming_client_recv() {
        let (_keep, rx) = mpsc::channel();
        let (tx, _keep) = mpsc::channel();
        let mut server = Server::create().unwrap();

        let password = "bob".to_string();
        let server_pass = Some(password.clone());

        let server_h = thread::spawn(move || {
            server.event_loop(tx, rx, server_pass).unwrap();
        });

        let connect = |password: String| {
            std::thread::spawn(move || {
                let mut stream = std::net::TcpStream::connect(SERVER_ADDR).unwrap();
                let mut first_response = [0; lib_spells::SERVER_HEADER.len()];
                stream.read_exact(&mut first_response).unwrap();
                assert_eq!(lib_spells::SERVER_HEADER, first_response);
                stream.write_all(&[password.len() as u8]).unwrap();
                stream.write_all(password.as_bytes()).unwrap();
                let mut buf = vec![];
                stream.read_to_end(&mut buf).unwrap();
            });
        };

        connect(password.clone());
        connect("not the password".into());
        dbg!(server_h.join().unwrap());
    }
}
