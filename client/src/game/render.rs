use crate::{cameras, game::replication, game::GameObject};
use bevy::{log, prelude::*};
use lib_spells::shared;

pub fn sys_render_players(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    query: Query<Entity, Added<shared::Player>>,
) {
    let player_mesh = meshes.add(Capsule3d::new(0.85, 1.75));
    let player_mat = materials.add(Color::BLUE);

    for player_entity in query.iter() {
        commands.entity(player_entity).insert(PbrBundle {
            mesh: player_mesh.clone(),
            material: player_mat.clone(),
            ..default()
        });
    }
}

pub fn sys_setup_player(mut commands: Commands, controlled_query: Query<Entity, With<replication::ControlledPlayer>>) {
    log::info!("setting up player");
    let mut camera = Camera3dBundle::default();
    camera.transform.translation = Vec3::new(0.0, 1.5, 0.0);

    commands.spawn((
        camera,
        cameras::follow_cam::FollowCamera::default(),
        GameObject,
    ));

    // tell our camera to follow the controlled player
    let controlled_player_entity = controlled_query.single();
    commands.entity(controlled_player_entity).insert(cameras::follow_cam::FollowCameraTarget);
}
