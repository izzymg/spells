use bevy::{
    prelude::*,
    window::{CursorGrabMode, PresentMode, PrimaryWindow},
};

use crate::input;

#[derive(States, Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum WindowContext {
    #[default]
    Menu,
    Play,
}
fn sys_lock_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = window_query.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
}

fn sys_unlock_cursor(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = window_query.single_mut();
    primary_window.cursor.grab_mode = CursorGrabMode::None;
}

fn sys_set_window_settings(mut window_query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = window_query.single_mut();
    primary_window.present_mode = PresentMode::Fifo;
}

fn escape_pressed(buttons: Res<input::ActionButtons>) -> bool {
    buttons.get_button_state(input::Action::Pause) == input::ButtonState::Pressed
}

fn primary_pressed(buttons: Res<input::ActionButtons>) -> bool {
    buttons.get_button_state(input::Action::Primary) == input::ButtonState::Pressed
}

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<WindowContext>();
        app.add_systems(Startup, sys_set_window_settings);
        app.add_systems(OnEnter(WindowContext::Play), sys_lock_cursor);
        app.add_systems(OnEnter(WindowContext::Menu), sys_unlock_cursor);
        app.add_systems(
            Update,
            sys_unlock_cursor
                .run_if(in_state(WindowContext::Play))
                .run_if(escape_pressed),
        );
        app.add_systems(
            Update,
            sys_lock_cursor
                .run_if(in_state(WindowContext::Play))
                .run_if(primary_pressed),
        );
    }
}
