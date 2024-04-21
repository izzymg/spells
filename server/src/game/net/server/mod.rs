/*! TCP server implementation for managing connected game clients */

pub mod packet;
mod connected_clients;
mod pending_clients;
mod tcp_stream;

use mio::net::TcpListener;
use mio::{Events, Interest, Poll};
pub use mio::Token;

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

    next_socket: usize,

    inc_tx: mpsc::Sender<Incoming>,
    out_rx: mpsc::Receiver<Outgoing>,
}

impl Server {
    pub fn create(
        inc_tx: mpsc::Sender<Incoming>,
        out_rx: mpsc::Receiver<Outgoing>,
    ) -> io::Result<Server> {
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
            next_socket: 1,

            inc_tx,
            out_rx,
        })
    }

    /// block on event look waiting for new clients, adding them by their token to a map of active cleint
    pub fn event_loop(&mut self) {
        let mut connected_clients = connected_clients::ConnectedClients::default();
        let mut pending_clients = pending_clients::PendingClients::default();

        loop {
            self.check_outgoing(&mut connected_clients);

            // accept new clients
            if let Some(mut new_client) = self.check_accept() {
                let token = Token(self.next_socket);
                self.next_socket += 1;
                new_client.register_to_poll(token, &mut self.poll).unwrap();
                log::info!("pending client: {}", new_client.ip_or_unknown());
                pending_clients.add_stream(token, new_client);
            }
            // drop expired pending clients
            for mut dead_client in pending_clients.remove_expired() {
                log::info!("dropping expired: {}", dead_client.ip_or_unknown());
                dead_client.deregister_from_poll(&mut self.poll).unwrap();
            }

            // check poll
            if let Err(poll_err) = self.poll.poll(&mut self.events, Some(MIN_TICK)) {
                log::warn!("poll error: {}", poll_err);
            }
            let client_events = self.events.iter().filter(|ev| ev.token() != SERVER_TOKEN);
            for ev in client_events {
                let client_token = ev.token();

                if let Some(new_client) = pending_clients.try_validate(client_token) {
                    // validate if it's incoming pending data
                    log::info!(
                        "connected client {}: {}",
                        client_token.0,
                        new_client.ip_or_unknown()
                    );
                    connected_clients.add(client_token, new_client);
                    self.inc_tx.send(Incoming::Joined(client_token)).unwrap();
                } else {
                    // receive from non-pending
                    match connected_clients.try_receive(client_token) {
                        Ok(Some(buf)) => {
                            self.inc_tx.send(Incoming::Data(client_token, buf)).unwrap();
                        }
                        Err(err) => {
                            log::info!("client {} errored out: {}", client_token.0, err);
                            connected_clients.remove(client_token);
                            self.inc_tx.send(Incoming::Left(client_token)).unwrap();
                        }
                        _ => {}
                    }
                }

                if ev.is_read_closed() {
                    log::info!("client {} quit", client_token.0);
                    connected_clients.remove(client_token);
                    self.inc_tx.send(Incoming::Left(client_token)).unwrap();
                }
            }
        }
    }

    /// receive data from the outside world to interact with clients (non-blocking)
    fn check_outgoing(&mut self, clients: &mut connected_clients::ConnectedClients) {
        match self.out_rx.try_recv() {
            Ok(outgoing) => match outgoing {
                Outgoing::Broadcast(data) => {
                    for err in clients.broadcast(&data).iter() {
                        log::info!("{}", err.error);
                        clients.remove(err.token);
                        self.inc_tx.send(Incoming::Left(err.token)).unwrap();
                    }
                }
                Outgoing::Kick(token) => {
                    clients.remove(token);
                    self.inc_tx.send(Incoming::Left(token)).unwrap();
                }
            },
            Err(err) if err == mpsc::TryRecvError::Disconnected => {
                panic!("receiver died: {}", err)
            }
            _ => {} // empty, we don't care
        }
    }

    /// check for new clients and accept them into pending
    /// client value gets dropped if we don't successfully pend them
    fn check_accept(&mut self) -> Option<tcp_stream::ClientStream> {
        let (stream, _) = match self.listener.accept() {
            Ok(s) => s,
            Err(_) => return None, // would block, just exit
        };
        let mut client = match tcp_stream::ClientStream::new(stream) {
            Ok(client) => client,
            Err(err) => {
                log::info!("failed to create client stream: {}", err);
                return None;
            }
        };
        if let Err(err) = client.write_header() {
            log::info!("failed to send server header: {}", err);
            return None;
        }
        Some(client)
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
    use mio::net::*;

    #[test]
    fn test_incoming_client_recv() {
        let (_keep, rx) = mpsc::channel();
        let (tx, _keep) = mpsc::channel();
        let mut server = Server::create(tx, rx).unwrap();
        // create a client stream
        // create a thread that blocks & fetches our clients
        // assert we grab the server header correctly
        // panic the thread if it doesn't process the client

        thread::spawn(move || {
            server.event_loop();
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

    #[test]
    #[ignore]
    fn test_tcp_things() {
        let (inc_tx, inc_rx) = mpsc::channel();
        let (_out_tx, out_rx) = mpsc::channel();
        let mut server = Server::create(inc_tx, out_rx).unwrap();
        let handle = std::thread::spawn(move || {
            server.event_loop();
        });

        let mut stream = TcpStream::connect(SERVER_ADDR.parse().unwrap()).unwrap();
        stream.set_nodelay(true).unwrap();
        stream
            .write_all(lib_spells::CLIENT_EXPECT.as_bytes())
            .unwrap();

        std::thread::spawn(move || loop {
            dbg!(inc_rx.recv().unwrap());
        });

        let sleep = Duration::from_millis(1000);
        println!("waiting {}s...", sleep.as_secs_f32());
        thread::sleep(sleep);
        stream.write_all("abcdefg".as_bytes()).unwrap();
        println!("write");
        stream.write_all("h".as_bytes()).unwrap();
        println!("write");
        handle.join().unwrap();
    }
}
