/// processes health entity update systems
use bevy::{
    ecs::{
        entity::Entity,
        schedule::IntoSystemConfigs,
        system::{Commands, Query},
    },
    hierarchy::DespawnRecursiveExt,
};

use super::Health;

/// Kills entities with no health, recursively (!!)
fn system_despawn_dead(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.hp <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn get_config() -> impl IntoSystemConfigs<()> {
    system_despawn_dead.into_configs()
}