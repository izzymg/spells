mod stream;
use bevy::{ecs::system::SystemId, log, prelude::*, tasks};
use std::sync::mpsc;

#[derive(Debug, Event)]
pub struct ClientInfoEvent(pub lib_spells::net::ClientInfo);

#[derive(Debug, Event)]
pub struct WorldStateEvent(pub lib_spells::net::WorldState);

#[derive(Debug, Event)]
pub struct DisconnectedEvent(pub Option<stream::ConnectionError>);

/// This should be a `NonSend` resource
#[derive(Debug)]
struct Connection {
    rx: mpsc::Receiver<stream::Incoming>,
    listen_handle: tasks::Task<stream::Result<()>>,
}

// Currently connecting
#[derive(Resource, Debug)]
struct Connecting {
    handle: tasks::Task<stream::Result<(stream::Connection, lib_spells::net::ClientInfo)>>,
}

/// Stores one shot connect system
#[derive(Debug, Resource)]
pub struct WorldConnection {
    pub connect_system: SystemId<(String, Option<String>)>,
}

impl WorldConnection {
    fn new(connect_system: SystemId<(String, Option<String>)>) -> Self {
        Self { connect_system }
    }
}

/// Check for disconnection and dispatch incoming data.
fn sys_connection(
    conn: Option<NonSendMut<Connection>>,
    mut ws_ev_w: EventWriter<WorldStateEvent>,
    mut err_ev_w: EventWriter<DisconnectedEvent>,
) {
    if let Some(connection) = conn {
        let connection = connection.into_inner();
        if let Some(res) = tasks::block_on(tasks::poll_once(&mut connection.listen_handle)) {
            log::info!("disconnected from world");
            err_ev_w.send(DisconnectedEvent(res.err()));
        }
        for data in connection.rx.try_iter() {
            match data {
                stream::Incoming::WorldState(state) => {
                    ws_ev_w.send(WorldStateEvent(state));
                }
            }
        }
    }
}

fn sys_connecting(
    connecting: Option<ResMut<Connecting>>,
    mut info_ev_w: EventWriter<ClientInfoEvent>,
    mut err_ev_w: EventWriter<DisconnectedEvent>,
) {
    if let Some(connecting) = connecting {
        let connecting = connecting.into_inner();
        let res = match tasks::block_on(tasks::poll_once(&mut connecting.handle)) {
            Some(res) => res,
            None => return,
        };

        match res {
            Ok((connection, info)) => {},
            Err(err) => {
                log::info!("connection failure: {}", err);
                err_ev_w.send(DisconnectedEvent(Some(err)));
            }
        }
    }
}

/// Handle requests to connect to a world.
fn sys_connect(In((addr, password)): In<(String, Option<String>)>, world: &mut World) {
    let handle = tasks::IoTaskPool::get()
        .spawn(async move { stream::get_connection(&addr.clone(), password.as_deref()) });

    world.insert_resource(Connecting { handle });
}

pub struct WorldConnectionPlugin;

impl Plugin for WorldConnectionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let connect_system = app.world.register_system(sys_connect);
        app.insert_resource(WorldConnection::new(connect_system));
        app.add_systems(Update, (sys_connection));
    }
}
