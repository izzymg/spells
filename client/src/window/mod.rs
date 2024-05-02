use crate::GameStates;
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PresentMode, PrimaryWindow},
};

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

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, sys_set_window_settings);
        app.add_systems(OnEnter(GameStates::LoadGame), sys_lock_cursor);
        app.add_systems(OnExit(GameStates::LoadGame), sys_unlock_cursor);
    }
}
