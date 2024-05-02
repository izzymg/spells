mod map;
mod replication;

use bevy::prelude::*;
use crate::GameStates;

#[derive(Component, Debug, Default)]
struct GameObject;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::controls::follow_cam::FollowCameraPlugin);
        app.add_systems(
            Update,
            (
                replication::sys_on_first_world_state.run_if(in_state(GameStates::LoadGame)),
                replication::sys_on_world_state.run_if(in_state(GameStates::Game)),
            ),
        );

        app.add_systems(OnEnter(GameStates::Game), map::sys_create_map);
        app.add_systems(OnExit(GameStates::Game), replication::sys_destroy_gos);

        app.insert_resource(replication::WorldGameEntityMap::default());
    }
}
