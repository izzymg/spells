mod main_menu;
mod styles;

use bevy::prelude::*;

use crate::{GameState, GameStates};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (main_menu::sys_build_menus, main_menu::sys_cleanup_menu_items));
        app.add_systems(
            Update,
            (styles::sys_button_interaction).run_if(|s: Res<GameState>| s.0.eq(&GameStates::Menu)),
        );
    }
}