mod socket;
use bevy::{app, log, prelude::*, tasks::IoTaskPool, utils::dbg};
use lib_spells::shared;
use std::sync::mpsc;

fn sys_create_state() -> shared::WorldState {
    shared::WorldState::default()
}

fn sys_update_component_world_state<T: Component + Into<shared::NeoState> + Clone>(
    In(mut world_state): In<shared::WorldState>,
    query: Query<(Entity, &T)>,
) -> shared::WorldState {
    query.iter().for_each(|(entity, comp)| {
        // clone is here so components can have uncopyable types like "timer"
        // however we should check performance of this and consider custom serialization of timer values if performance is bad
        world_state.update(entity.index(), comp.clone().into());
    });

    world_state
}

fn sys_broadcast_state(
    In(world_state): In<shared::WorldState>,
    mut sender: ResMut<ClientStreamSender>,
    mut exit_events: ResMut<Events<app::AppExit>>,
) -> shared::WorldState {
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

        IoTaskPool::get()
            .spawn(async move {
                log::debug!("client event loop task spawned");
                client_getter.event_loop(rx);
            })
            .detach();

        app.insert_resource(ClientStreamSender::new(tx));

        app.add_systems(
            FixedLast,
            sys_create_state
                .pipe(sys_update_component_world_state::<shared::Health>)
                .pipe(sys_update_component_world_state::<shared::Aura>)
                .pipe(sys_update_component_world_state::<shared::SpellCaster>)
                .pipe(sys_update_component_world_state::<shared::CastingSpell>)
                .pipe(sys_broadcast_state)
                .map(dbg),
        );
    }
}
