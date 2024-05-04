use crate::{game::replication, input};
use bevy::{log, prelude::*};

#[derive(Debug, Component, PartialEq, Default)]
pub struct WishDir(pub Vec3);

pub fn sys_setup_controls(
    mut commands: Commands,
    mut controlled_query: Query<Entity, With<replication::ControlledPlayer>>,
) {
    let entity = controlled_query.single_mut();
    commands.entity(entity).insert(WishDir::default());
}

/// Read current input axes and set the wish dir on the controlled player
pub fn sys_input_to_wish_dir(
    input_axes: Res<input::ActionAxes>,
    mut controlled_query: Query<&mut WishDir, With<replication::ControlledPlayer>>,
) {
    let mut dir = controlled_query
        .single_mut();
    dir.set_if_neq(WishDir(input_axes.get_movement_3d()));
}

/// Read the set wish dir on the controlled player and predict a new translation
pub fn sys_predict_player_pos(
    mut controlled_query: Query<(&mut Transform, &WishDir), With<replication::ControlledPlayer>>,
    time: Res<Time>,
) {
    let (mut controlled_trans, wish_dir) = controlled_query.single_mut();
    controlled_trans.translation += wish_dir.0 * time.delta_seconds();
}
