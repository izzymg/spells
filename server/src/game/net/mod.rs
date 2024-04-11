mod socket;
use bevy::{app, log, prelude::*, utils::dbg};
use lib_spells::serialization;
use std::sync::mpsc;

fn sys_create_state() -> serialization::WorldState {
    serialization::WorldState::default()
}

fn sys_update_component_world_state<T: Component + Into<serialization::NeoState> + Copy + Clone>(
    In(mut world_state): In<serialization::WorldState>,
    query: Query<(Entity, &T)>,
) -> serialization::WorldState {
    query.iter().for_each(|(entity, comp)| {
        world_state.update(entity.index(), (*comp).into());
    });

    world_state
}

fn sys_broadcast_state(
    In(world_state): In<serialization::WorldState>,
    mut sender: ResMut<ClientStreamSender>,
    mut exit_events: ResMut<Events<app::AppExit>>,
) -> serialization::WorldState {
    if !sender.send_data(
        world_state
            .serialize()
            .expect("world serialization failure"),
    ) {
        log::info!("client sender died, exiting");
        exit_events.send(app::AppExit);
    }
    world_state
}

// wraps a send channel
#[derive(bevy::ecs::system::Resource)]
pub struct ClientStreamSender(mpsc::Sender<Vec<u8>>);

impl ClientStreamSender {
    pub fn new(tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self(tx)
    }

    // Returns false if sending is now impossible (very bad)
    pub fn send_data(&mut self, data: Vec<u8>) -> bool {
        self.0.send(data).is_ok()
    }
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut client_getter = socket::client_server::ClientServer::create().unwrap();
        let (tx, rx) = mpsc::channel();
        client_getter.event_loop(rx);

        app.insert_resource(ClientStreamSender::new(tx));

        app.add_systems(
            FixedLast,
            sys_create_state.pipe(
                sys_update_component_world_state::<serialization::Health>
                    .pipe(sys_broadcast_state.map(dbg)),
            ),
        );
    }
}
