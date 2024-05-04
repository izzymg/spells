mod stream;
use crate::game::{controls, replication};
use bevy::{ecs::system::SystemId, log, prelude::*, tasks};
use std::time::Duration;

const PING_FREQUENCY: Duration = Duration::from_secs(4);

#[derive(Debug, Event)]
pub struct ConnectedEvent;

#[derive(Debug, Event)]
pub struct WorldStateEvent(pub lib_spells::net::WorldState);

#[derive(Debug, Event)]
pub struct DisconnectedEvent(pub Option<stream::ConnectionError>);

#[derive(Resource, Debug)]
pub struct Connection {
    connection: stream::Connection,
    ping_timer: Timer,
    pub client_info: lib_spells::net::ClientInfo,
}

impl Connection {
    pub fn get_latency(&self) -> Option<Duration> {
        self.connection.last_ping_rtt
    }
    fn new(conn: stream::Connection, client_info: lib_spells::net::ClientInfo) -> Self {
        Self {
            connection: conn,
            client_info,
            ping_timer: Timer::new(PING_FREQUENCY, TimerMode::Repeating),
        }
    }

    fn send_movement_input(&mut self, wish_dir: Vec3) -> stream::Result<()> {
        if wish_dir.length_squared() == 0.0 {
            self.connection.send_command(0, 0)?;
        }
        if wish_dir.z == -1.0 {
            self.connection.send_command(0, 1)?;
        }
        if wish_dir.x == 1.0 {
            self.connection.send_command(0, 2)?;
        }
        if wish_dir.z == 1.0 {
            self.connection.send_command(0, 3)?;
        }
        if wish_dir.x == -1.0 {
            self.connection.send_command(0, 4)?;
        }
        Ok(())
    }
}

pub fn sys_net_send_ping(time: Res<Time>, mut conn: ResMut<Connection>) -> stream::Result<()> {
    if conn.ping_timer.just_finished() {
        conn.connection.ping()?;
        log::debug!("ping");
    }
    conn.ping_timer.tick(time.delta());
    Ok(())
}

pub fn sys_net_send_movement(
    mut conn: ResMut<Connection>,
    wish_dir_query: Query<
        &controls::WishDir,
        (
            Changed<controls::WishDir>,
            With<replication::ControlledPlayer>,
        ),
    >,
) -> stream::Result<()> {
    wish_dir_query
        .iter()
        .try_for_each(|wd| conn.send_movement_input(wd.0))
}

fn sys_net_handle_error(
    In(err): In<stream::Result<()>>,
    mut dc_ev_w: EventWriter<DisconnectedEvent>,
) {
    if let Err(err) = err {
        log::warn!("caught network send error: {}", err);
        dc_ev_w.send(DisconnectedEvent(Some(err)));
    }
}

// Currently connecting
#[derive(Resource, Debug)]
struct Connecting {
    handle: tasks::Task<stream::Result<(stream::Connection, lib_spells::net::ClientInfo)>>,
}

/// Stores one shot connect system
#[derive(Debug, Resource)]
pub struct WorldConnectSys {
    pub connect_system: SystemId<(String, Option<String>)>,
}

impl WorldConnectSys {
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
    match connection.connection.read() {
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
            world.insert_resource(Connection::new(conn, client_info));
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

fn is_connection(conn: Option<Res<Connection>>) -> bool {
    conn.is_some()
}

pub struct WorldConnectionPlugin;

impl Plugin for WorldConnectionPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let connect_system = app.world.register_system(sys_connect);
        app.add_event::<ConnectedEvent>();
        app.add_event::<DisconnectedEvent>();
        app.add_event::<WorldStateEvent>();
        app.insert_resource(WorldConnectSys::new(connect_system));
        app.add_systems(
            Update,
            (
                sys_connection,
                sys_connecting,
                sys_net_send_ping.pipe(sys_net_handle_error).run_if(is_connection),
                sys_net_send_movement.pipe(sys_net_handle_error).run_if(is_connection),
            ),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_network_plugin() {
        let mut app = App::new();
        app.add_plugins((TaskPoolPlugin::default(), WorldConnectionPlugin));
        let connect_sys = app
            .world
            .get_resource_mut::<WorldConnectSys>()
            .unwrap()
            .connect_system;
        app.world
            .run_system_with_input(connect_sys, ("127.0.0.1:7776".into(), None))
            .unwrap();
        loop {
            let ev = app
                .world
                .get_resource_mut::<Events<ConnectedEvent>>()
                .unwrap();
            if ev.len() == 1 {
                break;
            }
            app.update();
        }
    }
}
