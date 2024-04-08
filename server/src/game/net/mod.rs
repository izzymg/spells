mod serialize;
mod socket;
use std::{sync::mpsc, thread};

use bevy::{app, log, prelude::*, utils::dbg};

use super::components;

fn sys_create_state(world: &mut World) -> serialize::WorldState {
    let mut state: serialize::WorldState = default();

    state.health = world
        .iter_entities()
        .filter_map(|e| {
            e.get::<components::Health>().and_then(|v| {
                Some(serialize::EntityHealth {
                    health: v.0,
                    entity: e.id(),
                })
            })
        })
        .collect();

    state.casters = world
        .iter_entities()
        .filter_map(|e| {
            e.get::<components::CastingSpell>().and_then(|v| {
                Some(serialize::EntityCaster {
                    max_timer: v.cast_timer.duration().as_millis(),
                    timer: v.cast_timer.elapsed().as_millis(),
                    spell_id: v.spell_id.get(),
                    entity: e.id(),
                })
            })
        })
        .collect();

    let mut aura_query = world.query::<(&Parent, &components::Aura)>();
    state.auras = world
        .iter_entities()
        .flat_map(|e| aura_query.get(world, e.id()))
        .map(|(p, a)| serialize::EntityAura {
            aura_id: a.id.get(),
            entity: p.get(),
            remaining: a.get_remaining_time().as_millis(),
        })
        .collect();

    state
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
                sys_create_state.pipe(sys_broadcast_state).map(dbg),
            );
    }
}
