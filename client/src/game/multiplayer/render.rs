use crate::{controls::cameras, events, render::terrain, replication};
use bevy::prelude::*;
use lib_spells::shared;

#[derive(Component)]
pub struct Cleanup;

pub fn sys_cleanup(mut commands: Commands, cleanup_query: Query<Entity, With<Cleanup>>, mut destroy_terrain_ev: EventWriter<events::DestroyTerrainEvent>) {
    for entity in cleanup_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    destroy_terrain_ev.send(events::DestroyTerrainEvent);
}

pub fn sys_create_map(mut terrain_event_send: EventWriter<events::GenerateTerrainEvent>) {
    let mut terrain = terrain::VoxelTerrain::default();
    for x in 0..50 {
        for y in 0..25 {
            terrain.0.push(terrain::Voxel(x, 0, y));
        }
    }
    terrain_event_send.send(events::GenerateTerrainEvent { terrain });
}

/// Add rendering to all new `Player` entities.
pub fn sys_add_player_rendering(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    query: Query<Entity, Added<shared::Player>>,
) {
    let player_mesh = meshes.add(Capsule3d::new(0.85, 1.75));
    let player_mat = materials.add(Color::BLUE);

    for player_entity in query.iter() {
        commands.entity(player_entity).insert((
            Cleanup,
            PbrBundle {
                mesh: player_mesh.clone(),
                material: player_mat.clone(),
                ..default()
            },
        ));
    }
}

/// Add a follow camera that follows the `PredictedPlayer`.
pub fn sys_follow_cam_predicted_player(
    mut commands: Commands,
    controlled_query: Query<Entity, Added<replication::PredictedPlayer>>,
) {
    let mut camera = Camera3dBundle::default();
    camera.transform.translation = Vec3::new(0.0, 1.5, 0.0);
    commands.spawn((
        camera,
        cameras::follow_cam::FollowCamera::default(),
        Cleanup,
    ));

    // tell our camera to follow the controlled player
    let controlled_player_entity = controlled_query.single();
    commands
        .entity(controlled_player_entity)
        .insert(cameras::follow_cam::FollowCameraTarget);
}
