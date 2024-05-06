pub mod controls;
mod map;
mod render;
pub mod replication;

use crate::{cameras, GameStates, SystemSets};
use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct GameObject;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(cameras::follow_cam::FollowCameraPlugin);

        // Replication
        app.insert_resource(replication::WorldGameEntityMap::default());
        app.insert_resource(replication::InputCache::default());
        app.add_systems(
            Update,
            (
                replication::sys_on_first_world_state.run_if(in_state(GameStates::LoadGame)),
                (
                    replication::sys_enqueue_movements,
                    replication::sys_predict_player_pos,
                )
                    .chain()
                    .run_if(in_state(GameStates::Game))
                    .before(SystemSets::NetSend)
                    .after(SystemSets::Controls),
                (
                    replication::sys_on_world_state,
                    replication::sys_sync_positions,
                )
                    .chain()
                    .run_if(in_state(GameStates::Game))
                    .after(SystemSets::NetFetch),
            ),
        );
        app.add_systems(OnExit(GameStates::Game), replication::sys_destroy_gos);
        // Map
        app.add_systems(OnEnter(GameStates::Game), map::sys_create_map);
        // Render
        app.add_systems(OnEnter(GameStates::Game), render::sys_setup_player);
        app.add_systems(
            Update,
            render::sys_render_players.run_if(in_state(GameStates::Game)),
        );

        // Controls
        app.add_systems(OnEnter(GameStates::Game), controls::sys_setup_controls);
        app.add_systems(
            Update,
            (controls::sys_input_to_wish_dir,)
                .chain()
                .in_set(SystemSets::Controls)
                .run_if(in_state(GameStates::Game)),
        );
    }
}
