mod connection;

use bevy::{app::Plugin, log};

use std::{sync::mpsc::{self, TryRecvError}, time::Duration};

const SERVER_ADDR: &str = "127.0.0.1:7776";

/// Non-send receiver of new World State data.
pub struct WorldConnectionRx(mpsc::Receiver<connection::WorldStateConnectionResult<Vec<u8>>>);

impl WorldConnectionRx {
    pub fn try_recv_data(&self) -> Result<Option<Vec<u8>>, connection::WorldConnectionError> {
        match self.0.try_recv() {
            Ok(res) => match res {
                Ok(res) => Ok(Some(res)),
                Err(err) => Err(err),
            },
            Err(err) => match err {
                TryRecvError::Disconnected => {
                    Err(connection::WorldConnectionError::ConnectionEnded)
                },
                // we don't care if it's empty
                _ => { Ok(None) }
            }
        }
    }
}

pub struct WorldConnectionPlugin;

impl Plugin for WorldConnectionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let (tx, rx) = mpsc::channel();

        std::thread::spawn(|| {
            let mut connection =
            connection::connect_retry(SERVER_ADDR, Duration::from_secs(3)).unwrap();
            connection.handshake().unwrap();
            match connection.listen(tx) {
                Ok(()) => println!("server listen finished"),
                Err(err) => println!("server listen died: {err}"),
            }
        });

        app.insert_non_send_resource(WorldConnectionRx(rx));
    }
}
