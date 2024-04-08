use crate::game::components;
use bevy::prelude::*;

use super::ServerSets;

/// Tick auras & remove expired
fn sys_tick_clean_auras(
    mut commands: Commands,
    mut query: Query<(Entity, &mut components::Aura)>,
    time: Res<Time>,
) {
    for (entity, mut effect) in query.iter_mut() {
        effect.duration.tick(time.delta());
        if effect.duration.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Kills entities with no health, recursively (!!)
fn sys_despawn_dead(mut commands: Commands, query: Query<(Entity, &components::Health)>) {
    for (entity, health) in query.iter() {
        if health.hp <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct EntityProcessingPlugin;

impl Plugin for EntityProcessingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            FixedUpdate,
            (sys_tick_clean_auras, sys_despawn_dead).in_set(ServerSets::EffectProcessing),
        );
    }
}
