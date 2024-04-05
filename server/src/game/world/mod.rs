use std::{sync::mpsc, thread};

/// snapshots of world
use bevy::{
    app::{FixedLast, Plugin},
    ecs::system::{In, Query, ResMut, Resource},
    prelude::IntoSystem,
    utils::dbg,
};

use crate::game::spells::casting;

use super::socket;

#[derive(Debug)]
pub struct CasterState {
    pub timer: u128,
    pub max_timer: u128,
    pub spell_id: usize,
}

#[derive(Debug, Default)]
pub struct WorldState {
    pub casters: Vec<CasterState>,
}

fn create_state_sys() -> WorldState {
    WorldState::default()
}

fn state_casters_sys(
    In(mut world_state): In<WorldState>,
    query: Query<&casting::CastingSpell>,
) -> WorldState {
    world_state.casters = query
        .iter()
        .map(|caster| CasterState {
            max_timer: caster.cast_timer.duration().as_millis().into(),
            spell_id: caster.spell_id.get(),
            timer: caster.cast_timer.elapsed().as_millis().into(),
        })
        .collect();
    world_state
}

fn broadcast_state_to_clients_sys(
    In(world_state): In<WorldState>,
    broadcaster: ResMut<ClientBroadcaster>,
) -> WorldState {
    broadcaster.0.send("FAKE STATE\n".into()).unwrap();
    world_state
}

#[derive(Resource)]
pub struct ClientBroadcaster(mpsc::Sender<String>);

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut client_getter = socket::client_server::ClientServer::create().unwrap();
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            client_getter.block_get_client(rx);
        });

        app.insert_resource(ClientBroadcaster(tx)).add_systems(
            FixedLast,
            create_state_sys.pipe(
                state_casters_sys
                    .pipe(broadcast_state_to_clients_sys)
                    .map(dbg),
            ),
        );
    }
}
