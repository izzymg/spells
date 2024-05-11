mod main_menu_control;
mod main_menu_view;
use super::GameStates;
use bevy::prelude::*;

pub(super) struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<main_menu_control::ConnectEvent>();
        app.init_resource::<main_menu_control::ConnectionStatus>();
        app.add_systems(
            OnEnter(GameStates::MainMenu),
            main_menu_view::sys_create_main_menu,
        );
        app.add_systems(
            OnExit(GameStates::MainMenu),
            main_menu_view::sys_destroy_main_menu,
        );

        app.add_systems(
            Update,
            (
                main_menu_view::sys_on_click_connect_btn,
                main_menu_view::sys_update_status_text,
                main_menu_control::sys_on_menu_connect,
                main_menu_control::sys_on_connected,
                main_menu_control::sys_on_disconnected,
            )
                .run_if(in_state(GameStates::MainMenu)),
        );
    }
}
