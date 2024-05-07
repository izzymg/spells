/*! Maps input onto a  `PredictedPlayer` */ 
use crate::{game::replication, input, GameStates, SystemSets};
use bevy::prelude::*;

/// Read current input axes and set the `WishDir` on the controlled player.
pub fn sys_input_to_wish_dir(
    input_axes: Res<input::ActionAxes>,
    mut controlled_query: Query<&mut replication::WishDir, With<replication::PredictedPlayer>>,
) {
    let mut dir = controlled_query.single_mut();
    dir.set_if_neq(replication::WishDir(input_axes.get_movement_3d()));
}

pub struct PredictedPlayerControlsPlugin;

impl Plugin for PredictedPlayerControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (sys_input_to_wish_dir)
                .chain()
                .in_set(SystemSets::Controls)
                .run_if(in_state(GameStates::Game)),
        );
    }
}
