mod socket;
use super::components;
use bevy::{app, log, prelude::*, utils::dbg};
use lib_spells::serialization;
use std::{sync::mpsc, thread};

fn sys_create_state(world: &mut World) -> serialization::WorldState {
    let mut state: serialization::WorldState = default();

    // health
    for (entity, hp) in world.query::<(Entity, &components::Health)>().iter(world) {
        state.update(
            entity.index(),
            serialization::EntityState::default()
                .with_health(serialization::EntityHealth { health: hp.0 }),
        );
    }

    // spell casts
    for (entity, cast) in world
        .query::<(Entity, &components::CastingSpell)>()
        .iter(world)
    {
        state.update(
            entity.index(),
            serialization::EntityState::default().with_casting_spell(
                serialization::EntityCastingSpell {
                    max_timer: cast.cast_timer.duration().as_millis() as u64,
                    timer: cast.cast_timer.elapsed().as_millis() as u64,
                    spell_id: cast.spell_id.get(),
                },
            ),
        );
    }

    // casters
    for (entity, _) in world
        .query::<(Entity, &components::SpellCaster)>()
        .iter(world)
    {
        state.update(
            entity.index(),
            serialization::EntityState::default()
                .with_spell_caster(serialization::EntitySpellCaster),
        );
    }

    // auras
    for (parent, aura) in world.query::<(&Parent, &components::Aura)>().iter(world) {
        state.update(
            parent.get().index(),
            serialization::EntityState::default().with_aura(serialization::EntityAura {
                aura_id: aura.id.get(),
                remaining: aura.get_remaining_time().as_millis() as u64,
            }),
        );
    }

    state
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
