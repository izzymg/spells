use crate::{controls::free_cam, render, GameState, GameStates};
use bevy::prelude::*;

fn sys_spawn(
    mut commands: Commands,
    mut terrain_event_send: EventWriter<render::GenerateTerrainEvent>,
) {
    commands.spawn((
        Camera3dBundle::default(),
        free_cam::FreeCamera::default(),
    ));

    let terrain = render::VoxelTerrain(vec![render::Voxel(0, 0, 0)]);
    terrain_event_send.send(render::GenerateTerrainEvent { terrain });
}

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(free_cam::FreeCameraPlugin);
        app.world.get_resource_mut::<GameState>().unwrap().0 = GameStates::Game;
        app.add_systems(Startup, sys_spawn);
    }
}
