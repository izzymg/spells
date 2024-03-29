use bevy::ecs::{component::Component, entity::Entity, query::With, system::{Commands, Query}};

pub fn cleanup_system<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).remove::<T>();
    }
}