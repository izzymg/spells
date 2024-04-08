mod serialize;
mod socket;
use std::{sync::mpsc, thread};

use bevy::{app, prelude::*, utils::dbg};

use crate::game::components;

fn sys_create_state() -> serialize::WorldState {
    serialize::WorldState::default()
}

fn sys_get_casters(
    In(mut world_state): In<serialize::WorldState>,
    query: Query<&components::CastingSpell>,
) -> serialize::WorldState {
    world_state.casters = query
        .iter()
        .map(|c| serialize::CasterState::from(c))
        .collect();
    world_state
}

fn sys_broadcast_state(
    In(world_state): In<serialize::WorldState>,
    mut sender: ResMut<ClientStreamSender>,
    mut exit_events: ResMut<Events<app::AppExit>>,
) -> serialize::WorldState {
    if !sender.send_data(
        world_state
            .serialize()
            .expect("world serialization failure"),
    ) {
        println!("Client sender died, exiting");
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
        !self.0.send(data).is_err()
    }
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut client_getter = socket::client_server::ClientServer::create().unwrap();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            client_getter.event_loop(rx);
        });

        app.insert_resource(ClientStreamSender::new(tx))
            .add_systems(
                FixedLast,
                sys_create_state.pipe(sys_get_casters.pipe(sys_broadcast_state).map(dbg)),
            );
    }
}
