use crate::ui::widgets;
use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct LoadingUI;

pub fn sys_create_loading_ui(mut commands: Commands) {
    commands.spawn((LoadingUI, Camera2dBundle::default()));
    commands
        .spawn((LoadingUI, NodeBundle::default()))
        .with_children(|c| {
            c.spawn(widgets::title_text("LOADING".into()));
        });
}

pub fn sys_destroy_loading_ui(mut commands: Commands, ui_query: Query<Entity, With<LoadingUI>>) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
