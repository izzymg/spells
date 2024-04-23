/*! TCP server implementation for managing connected game clients */

pub mod packet;
mod connection_manager;

use mio::net::TcpListener;
pub use mio::Token;
use mio::{Events, Interest, Poll};

use std::io;
use std::sync::mpsc;
use std::time::Duration;

use bevy::log;

const SERVER_TOKEN: Token = Token(0);
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
            .register(&mut listener, SERVER_TOKEN, Interest::READABLE)?;
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
    ) -> io::Result<()> {
        let mut manager = connection_manager::ConnectionManager::new(inc_tx, out_rx);

        let mut next_socket = 0_usize;
        let mut next_token = || {
            let token = Token(next_socket);
            next_socket += 1;
            token
        };

        loop {
            self.poll
                .poll(&mut self.events, Some(MIN_TICK))
                .expect("poll died");

            manager.tick();
            manager.collect_dead(|dead| {
                log::debug!("deregister dead: {}", dead.ip_or_unknown());
                self.poll
                    .registry()
                    .deregister(&mut dead.into_inner())
                    .expect("poll dead");
            });
            for ev in self.events.iter() {
                match ev.token() {
                    // new connections inc
                    SERVER_TOKEN => loop {
                        let (mut stream, addr) = match self.listener.accept() {
                            Ok((stream, addr)) => (stream, addr),
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(e) => return Err(e),
                        };

                        log::info!("got new connection from: {}", addr);
                        let new_token = next_token();
                        self.poll
                            .registry()
                            .register(
                                &mut stream,
                                new_token,
                                Interest::READABLE.add(Interest::WRITABLE),
                            )
                            .unwrap();
                        manager.manage_stream(new_token, connection_manager::tcp_stream::ClientStream::new(stream));
                    },
                    // this is a managed client
                    _ => manager.handle_event(ev)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        sync::mpsc,
        thread,
    };

    use super::*;

    #[test]
    fn test_incoming_client_recv() {
        let (_keep, rx) = mpsc::channel();
        let (tx, _keep) = mpsc::channel();
        let mut server = Server::create().unwrap();
        // create a client stream
        // create a thread that blocks & fetches our clients
        // assert we grab the server header correctly
        // panic the thread if it doesn't process the client

        thread::spawn(move || {
            server.event_loop(tx, rx);
        });

        let connect = || {
            // use a std stream so it blocks
            let mut stream = std::net::TcpStream::connect(SERVER_ADDR).unwrap();
            let mut first_response = [0; lib_spells::SERVER_HEADER.len()];
            stream.read_exact(&mut first_response).unwrap();
            assert_eq!(lib_spells::SERVER_HEADER.as_bytes(), first_response);
        };

        connect();
        connect();
    }
}
