use crate::{controls, game::GameObject};
use bevy::log;
use bevy::prelude::*;

pub fn sys_create_map(
    mut commands: Commands) {

    log::info!("creating map");

    commands.spawn((
            Camera3dBundle::default(),
            controls::follow_cam::FollowCamera::default(),
            GameObject,
    ));
}

