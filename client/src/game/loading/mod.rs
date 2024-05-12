use super::GameStates;
use crate::{events, ui::widgets};
use bevy::{log, prelude::*};

#[derive(Component, Debug, Default)]
struct LoadingUI;

fn sys_create_loading_ui(mut commands: Commands) {
    commands.spawn((LoadingUI, Camera2dBundle::default()));
    commands
        .spawn((LoadingUI, NodeBundle::default()))
        .with_children(|c| {
            c.spawn(widgets::title_text("LOADING".into()));
        });
}

fn sys_on_replication_complete(mut ns: ResMut<NextState<GameStates>>) {
    log::debug!("caught replication, moving state");
    ns.set(GameStates::Game);
}

fn sys_destroy_loading_ui(mut commands: Commands, ui_query: Query<Entity, With<LoadingUI>>) {
    for entity in ui_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameStates::Loading), sys_create_loading_ui);
        app.add_systems(Update, sys_on_replication_complete.run_if(on_event::<events::ReplicationCompleted>()).run_if(in_state(GameStates::Loading)));
        app.add_systems(OnExit(GameStates::Loading), sys_destroy_loading_ui);
    }
}
