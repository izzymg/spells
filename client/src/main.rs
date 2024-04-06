use std::{
    error::Error,
    io,
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use bevy::{
    app::{AppExit, Update},
    ecs::{
        event::Events,
        system::{NonSend, ResMut},
    },
    DefaultPlugins,
};

pub mod world_connection;

const SERVER_ADDR: &str = "127.0.0.1:7776";
fn check_world_server_data_system(
    fetch: NonSend<ServerStateReceiver>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    match fetch.0.try_recv() {
        Ok(msg) => match msg {
            Ok(world_state) => {
                println!("NEW WORLD STATE");
            }
            Err(err) => {
                println!("game loop exiting: {}", err);
                app_exit_events.send(AppExit);
            }
        },
        Err(err) => match err {
            mpsc::TryRecvError::Disconnected => {
                println!("game loop exiting: world state listener disconnected");
                app_exit_events.send(AppExit);
            }
            _ => {}
        },
    }
}

pub struct ServerStateReceiver(Receiver<world_connection::WorldStateConnectionResult<Vec<u8>>>);

fn main() -> Result<(), Box<dyn Error>> {
    {
        let (tx, rx) = mpsc::channel();

        let mut connection = world_connection::connect_retry(SERVER_ADDR, Duration::from_secs(3))?;
        connection.handshake()?;
        std::thread::spawn(move || match connection.listen(tx) {
            Ok(()) => println!("server listen finished"),
            Err(err) => println!("server listen died: {err}"),
        });

        bevy::app::App::new()
            .add_plugins(DefaultPlugins)
            .insert_non_send_resource(ServerStateReceiver(rx))
            .add_systems(Update, check_world_server_data_system)
            .run();
        Ok(())
    }
}
