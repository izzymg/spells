use std::time::Duration;

use bevy::{
    app::{FixedUpdate, Plugin, Update},
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

use self::{shield::StatusShield, ticking_hp::StatusTickingHP};

mod resource;
mod shield;
mod ticking_hp;

/// Possible status effect types
enum StatusEffectType {
    TickingHP,
    Shield,
}

///T his entity has a status effect, we can look up its complex data
#[derive(Component)]
pub struct StatusEffect {
    pub id: usize,
    pub duration: Timer,
}

impl StatusEffect {
    pub fn get_remaining_time(&self, max_duration: Duration) -> Duration {
        max_duration - self.duration.elapsed()
    }
}



/// Tick status effects & remove expired
fn tick_status_effects_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut StatusEffect)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.duration.tick(time.delta());
        if effect.duration.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Request to add a status effect child to the given entity
#[derive(Event, Debug)]
pub struct AddStatusEffectEvent {
    pub status_id: usize,
    pub target_entity: Entity,
}

/// Request to drop a status effect child from the given entity
#[derive(Event, Debug)]
pub struct RemoveStatusEffectEvent {
    pub status_id: usize,
    pub target_entity: Entity,
}

/// Process an add status effect event
fn add_status_effect_system(
    mut ev_r: EventReader<AddStatusEffectEvent>,
    mut commands: Commands,
    status_system: resource::StatusEffectSystem,
) {
    for ev in ev_r.read() {
        // look up status
        if let Some(status_data) = status_system.get_status_effect_data(ev.status_id) {
        // spawn base status effect
        let base_entity = commands
            .spawn((StatusEffect {
                id: ev.status_id,
                duration: Timer::new(status_data.duration, TimerMode::Once)
            },))
            .id();

        // add status effect types
        match status_data.status_type {
            StatusEffectType::TickingHP => commands.entity(base_entity).insert(StatusTickingHP::new()),
            StatusEffectType::Shield => commands.entity(base_entity).insert(StatusShield::new()),
        };

        // parent
        commands.entity(ev.target_entity).add_child(base_entity);

        log::debug!("added aura ID {} ({:?})", ev.status_id, base_entity)
        }
    }
}

/// Process a remove status effect event
fn remove_status_effect_system(
    mut ev_r: EventReader<RemoveStatusEffectEvent>,
    mut commands: Commands,
    child_query: Query<&Children>,
    status_effect_query: Query<&StatusEffect>,
) {
    'event_processing: for ev in ev_r.read() {
        // find children of entity
        if let Ok(children) = child_query.get(ev.target_entity) {
            for &child in children.iter() {
                // for each child grab status
                if let Ok(status) = status_effect_query.get(child) {
                    if status.id == ev.status_id {
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

pub struct StatusEffectPlugin;

impl Plugin for StatusEffectPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<AddStatusEffectEvent>()
        .add_event::<RemoveStatusEffectEvent>()
        .insert_resource(resource::get_resource())
        .add_systems(
            Update,
            (
                tick_status_effects_system,
                add_status_effect_system,
                remove_status_effect_system,
                ticking_hp::ticking_damage_system,
                shield::status_shield_system,
            ),
        );
    }
}
