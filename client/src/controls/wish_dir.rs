use crate::{input, SystemSets};
use bevy::prelude::*;

#[derive(PartialEq, Resource, Debug, Copy, Clone, Default)]
pub struct WishDir(pub Vec3);

pub fn sys_update_wish_dir(
    input_axes: Res<input::ActionAxes>,
    mut wish_dir: ResMut<WishDir>,
) {
    wish_dir.set_if_neq(WishDir(input_axes.get_movement_3d()));
}

pub struct WishDirPlugin;

impl Plugin for WishDirPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WishDir>();
        app.add_systems(
            Update,
            (sys_update_wish_dir)
                .chain()
                .in_set(SystemSets::Controls)
        );
    }
}
