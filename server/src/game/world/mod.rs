use std::{sync::mpsc, thread};

/// snapshots of world
use bevy::{
    app::{AppExit, FixedLast, Plugin},
    ecs::{event::Events, system::{In, Query, ResMut}},
    prelude::IntoSystem,
    utils::dbg,
};

mod sender;
use self::sender::ClientStreamSender;

use super::{serialize, socket, spells};

fn create_state_sys() -> serialize::WorldState {
    serialize::WorldState::default()
}

fn state_casters_sys(
    In(mut world_state): In<serialize::WorldState>,
    query: Query<&spells::CastingSpell>,
) -> serialize::WorldState {
    world_state.casters = query.iter().map(|c| serialize::CasterState::from(c)).collect();
    world_state
}

fn broadcast_state_to_clients_sys(
    In(world_state): In<serialize::WorldState>,
    mut sender: ResMut<ClientStreamSender>,
    mut exit_events: ResMut<Events<AppExit>>,
) -> serialize::WorldState {
    if !sender.send_data(world_state.serialize().expect("world serialization failure")) {
        println!("Client sender died, exiting");
        exit_events.send(AppExit);
    }
    world_state
}


pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut client_getter = socket::client_server::ClientServer::create().unwrap();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            client_getter.event_loop(rx);
        });

        app.insert_resource(ClientStreamSender::new(tx)).add_systems(
            FixedLast,
            create_state_sys.pipe(
                state_casters_sys
                    .pipe(broadcast_state_to_clients_sys)
                    .map(dbg),
            ),
        );
    }
}
