use std::time::Duration;

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        system::{Commands, Query, Res},
    },
    hierarchy::{BuildChildren, Children, DespawnRecursiveExt},
    log,
    time::{Time, Timer, TimerMode},
};

use self::{shield::{ShieldDamageEvent, StatusShield}, ticking_hp::AuraTickingHealth};

mod resource;
pub mod shield;
pub mod ticking_hp;

/// Possible aura types
enum StatusEffectType {
    TickingHP,
    Shield,
}

///T his entity has a aura, we can look up its complex data
#[derive(Component)]
pub struct Aura {
    pub id: usize,
    pub duration: Timer,
}

impl Aura {
    pub fn get_remaining_time(&self, max_duration: Duration) -> Duration {
        max_duration - self.duration.elapsed()
    }
}

/// Tick auras & remove expired
fn tick_auras_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Aura)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.duration.tick(time.delta());
        if effect.duration.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Request to add a aura child to the given entity
#[derive(Event, Debug)]
pub struct AddAuraEvent {
    pub aura_id: usize,
    pub target_entity: Entity,
}

/// Request to drop a aura child from the given entity
#[derive(Event, Debug)]
pub struct RemoveAuraEvent {
    pub aura_id: usize,
    pub target_entity: Entity,
}

/// Process an add aura event
fn add_status_effect_system(
    mut ev_r: EventReader<AddAuraEvent>,
    mut commands: Commands,
    auras_db: resource::AuraSysResource,
) {
    for ev in ev_r.read() {
        // look up status
        if let Some(aura_data) = auras_db.get_status_effect_data(ev.aura_id) {
            // spawn base aura
            let base_entity = commands
                .spawn((Aura {
                    id: ev.aura_id,
                    duration: Timer::new(aura_data.duration, TimerMode::Once),
                },))
                .id();

            // add aura types
            match aura_data.status_type {
                StatusEffectType::TickingHP => {
                    commands.entity(base_entity).insert(AuraTickingHealth::new())
                }
                StatusEffectType::Shield => commands
                    .entity(base_entity)
                    .insert(StatusShield::new(aura_data.base_multiplier)),
            };

            // parent
            commands.entity(ev.target_entity).add_child(base_entity);

            log::debug!("added aura ID {} ({:?})", ev.aura_id, base_entity)
        }
    }
}

/// Process a remove aura event
fn remove_aura_system(
    mut ev_r: EventReader<RemoveAuraEvent>,
    mut commands: Commands,
    child_query: Query<&Children>,
    status_effect_query: Query<&Aura>,
) {
    'event_processing: for ev in ev_r.read() {
        // find children of entity
        if let Ok(children) = child_query.get(ev.target_entity) {
            for &child in children.iter() {
                // for each child grab status
                if let Ok(status) = status_effect_query.get(child) {
                    if status.id == ev.aura_id {
                        // drop one instance of this status
                        commands.entity(child).despawn_recursive();
                        log::debug!("removed aura ID {} ({:?})", status.id, child);
                        continue 'event_processing;
                    }
                }
            }
        }
    }
}

pub struct AuraPlugin;

impl Plugin for AuraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_event::<AddAuraEvent>()
            .add_event::<RemoveAuraEvent>()
            .add_event::<ShieldDamageEvent>()
            .insert_resource(resource::get_resource())
            .add_systems(
                FixedUpdate,
                (
                    tick_auras_system,
                    add_status_effect_system,
                    remove_aura_system,
                    ticking_hp::ticking_damage_system,
                    shield::shield_damage_system,
                ),
            );
    }
}
