use crate::{GameState, GameStates};
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PresentMode, PrimaryWindow},
};

/// Set cursor to unlocked when the game state changes to menu
fn sys_change_cursor_mode(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    game_state: Res<GameState>,
) {
    if !game_state.is_changed() {
        return;
    }

    let mut primary_window = window_query.single_mut();
    match game_state.0 {
        GameStates::Menu => {
            primary_window.cursor.grab_mode = CursorGrabMode::None;
        }
        GameStates::Game => {
            primary_window.cursor.grab_mode = CursorGrabMode::Locked;
        }
    }
}

fn sys_set_window_settings(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = window_query.single_mut();
    primary_window.present_mode = PresentMode::Fifo;
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_set_window_settings);
        app.add_systems(Update, sys_change_cursor_mode);
    }
}
