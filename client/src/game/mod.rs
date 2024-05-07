pub mod controls;
mod map;
mod render;
pub mod replication;

use crate::{cameras, GameStates};
use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct GameObject;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(cameras::follow_cam::FollowCameraPlugin);

        // Replication
        app.add_plugins(replication::ReplicationPlugin);
        // Map
        app.add_systems(OnEnter(GameStates::Game), map::sys_create_map);
        // Render
        app.add_systems(
            OnEnter(GameStates::Game),
            render::sys_follow_cam_predicted_player,
        );
        app.add_systems(
            Update,
            render::sys_add_player_rendering.run_if(in_state(GameStates::Game)),
        );

        // Controls
        app.add_plugins(controls::PredictedPlayerControlsPlugin);
    }
}
