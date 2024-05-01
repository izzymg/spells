mod stream;
use bevy::{ecs::system::SystemId, log, prelude::*, tasks};

#[derive(Debug, Event)]
pub struct ConnectedEvent;

#[derive(Debug, Event)]
pub struct WorldStateEvent(pub lib_spells::net::WorldState);

#[derive(Debug, Event)]
pub struct DisconnectedEvent(pub Option<stream::ConnectionError>);

#[derive(Resource, Debug)]
pub struct Connection {
    conn: stream::Connection,
    pub client_info: lib_spells::net::ClientInfo,
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
fn sys_connection(world: &mut World) {
    let connection = match world.get_resource_mut::<Connection>() {
        Some(connection) => connection.into_inner(),
        None => return,
    };
    match connection.conn.read() {
        Ok(states) => {
            for state in states {
                world
                    .get_resource_mut::<Events<WorldStateEvent>>()
                    .unwrap()
                    .send(WorldStateEvent(state));
            }
        }
        Err(err) => {
            world
                .get_resource_mut::<Events<DisconnectedEvent>>()
                .unwrap()
                .send(DisconnectedEvent(Some(err)));
            world.remove_resource::<Connection>();
            log::debug!("removed connection resource");
        }
    }
}

fn sys_connecting(world: &mut World) {
    let mut connecting = match world.get_resource_mut::<Connecting>() {
        Some(connecting) => connecting,
        None => return,
    };
    let res = match tasks::block_on(tasks::poll_once(&mut connecting.handle)) {
        Some(res) => res,
        None => return,
    };

    world.remove_resource::<Connecting>().unwrap();

    match res {
        Ok((conn, client_info)) => {
            log::debug!("inserted connection resource");
            world
                .get_resource_mut::<Events<ConnectedEvent>>()
                .unwrap()
                .send(ConnectedEvent);
            world.insert_resource(Connection { conn, client_info });
        }
        Err(err) => {
            log::info!("connection failure: {}", err);
            world
                .get_resource_mut::<Events<DisconnectedEvent>>()
                .unwrap()
                .send(DisconnectedEvent(Some(err)));
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
        app.add_event::<ConnectedEvent>();
        app.add_event::<DisconnectedEvent>();
        app.add_event::<WorldStateEvent>();
        app.insert_resource(WorldConnection::new(connect_system));
        app.add_systems(Update, (sys_connection, sys_connecting));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_network_plugin() {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), WorldConnectionPlugin));
        let connect_sys = app.world.get_resource_mut::<WorldConnection>().unwrap().connect_system;
        app.world.run_system_with_input(connect_sys, ("127.0.0.1:7776".into(), None)).unwrap();
        loop {
            let ev = app.world.get_resource_mut::<Events<ConnectedEvent>>().unwrap();
            if ev.len() == 1 {
                break;
            }
            app.update();
        }
    }
}
