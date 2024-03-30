use std::time::Duration;

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        system::{Commands, Query, Res},
    },
    hierarchy::{BuildChildren, Children, Parent},
    log,
    time::{Time, Timer},
};

use self::resource::AuraList;

mod resource;


#[derive(Event)]
pub struct AddAuraEvent {
    pub entity: Entity,
    pub aura_id: usize,
}

#[derive(Event)]
pub struct RemoveAuraEvent {
    pub entity: Entity,
    pub aura_id: usize,
}

#[derive(Component)]
struct Aura {
    aura_id: usize,
    timer: Timer,
}

impl Aura {
    fn get_time_remaining(&self, duration: Duration) -> Duration {
        duration - self.timer.elapsed()
    }
}

enum AuraType {
    DOT,
    SHIELD,
    THORNS,
}

struct AuraData {
    pub name: String,
    pub duration: Duration,
    pub aura_type: AuraType,
}

impl AuraData {
    fn new(name: String, duration: u64, aura_type: AuraType) -> AuraData {
        AuraData {
            name,
            duration: Duration::from_millis(duration),
            aura_type,
        }
    }
}

// spawn auras as children of given entity
fn on_add_aura_event_system(
    mut commands: Commands,
    mut ev_r: EventReader<AddAuraEvent>,
    aura_list: Res<AuraList>,
) {
    for ev in ev_r.read() {
        // find relevant aura
        if let Some(aura_data) = aura_list.get_aura_data(ev.aura_id) {
            // create aura entity
            let aura = commands
                .spawn(Aura {
                    aura_id: ev.aura_id,
                    timer: Timer::new(aura_data.duration, bevy::time::TimerMode::Once),
                })
                .id();

            // aura is child of parent entity
            commands.entity(ev.entity).push_children(&[aura]);
        } else {
            log::error!("no aura at id {}", ev.aura_id)
        }
    }
}

// drop aura by ID on given entity
fn on_remove_aura_event_system(
    mut commands: Commands,
    mut ev_r: EventReader<RemoveAuraEvent>,
    query: Query<&Children>,
    query_auras: Query<&Aura>,
) {
    for ev in ev_r.read() {
        // get all child entities of our parent
        for &child in query.get(ev.entity).unwrap().iter() {
            // for every aura child
            if let Ok(aura) = query_auras.get(child) {
                // find aura and remove match
                if aura.aura_id == ev.aura_id {
                    commands.entity(child).despawn();
                    log::debug!("removed aura {} from {:?}", ev.aura_id, ev.entity)
                }
            }
        }
    }
}

// tick aura timers and send remove event when timer is done
fn aura_tick_system(
    time: Res<Time>,
    mut ev_w: EventWriter<RemoveAuraEvent>,
    mut query: Query<(&Parent, &mut Aura)>,
) {
    for (parent, mut aura) in query.iter_mut() {
        // tick all auras
        aura.timer.tick(time.delta());

        if aura.timer.finished() {
            // remove aura id from the parent
            ev_w.send(RemoveAuraEvent {
                entity: parent.get(),
                aura_id: aura.aura_id,
            });
        }
    }
}

fn debug_aura_system(aura_list: Res<AuraList>, query: Query<(Entity, &Aura)>) {
    for (entity, aura) in query.iter() {

        let aura_data = aura_list.get_aura_data(aura.aura_id).unwrap();

        log::debug!(
            "{:?} has aura ({}: {}) ({}/{} s)",
            entity,
            aura.aura_id,
            aura_data.name,
            aura.get_time_remaining(aura_data.duration).as_secs(),
            aura_data.duration.as_secs(),
        );
    }
}

pub struct AurasPlugin;
impl Plugin for AurasPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .insert_resource(resource::get_aura_list_resource())
            .add_event::<AddAuraEvent>()
            .add_event::<RemoveAuraEvent>()
            .add_systems(
                FixedUpdate,
                (
                    aura_tick_system,
                    debug_aura_system,
                    on_add_aura_event_system,
                    on_remove_aura_event_system
                ),
            );
    }
}
