use crate::{game::replication, input};
use bevy::{log, prelude::*};

pub fn sys_player_movement_input(
    input_axes: Res<input::ActionAxes>,
    mut controlled_query: Query<&mut Transform, With<replication::ControlledPlayer>>,
    time: Res<Time>,
) {
    let mut controlled_trans = controlled_query.single_mut();
    
    let wish_dir = input_axes.get_movement_3d();
    controlled_trans.translation += wish_dir * time.delta_seconds();
}
