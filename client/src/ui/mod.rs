mod game_view;
mod main_menu_control;
mod main_menu_view;
mod widgets;

use bevy::prelude::*;

use crate::GameStates;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameStates::MainMenu),
            main_menu_view::sys_create_main_menu,
        );
        app.add_systems(
            OnExit(GameStates::MainMenu),
            main_menu_view::sys_cleanup_main_menu,
        );
        app.insert_resource(main_menu_control::ConnectionStatus::default());
        app.add_event::<main_menu_control::ConnectEvent>();
        app.add_systems(
            Update,
            (
                main_menu_view::sys_on_click_connect_btn,
                main_menu_view::sys_update_status_text,
                main_menu_control::sys_menu_connect_ev,
                main_menu_control::sys_update_connection_status,
                widgets::sys_button_interaction,
                widgets::sys_text_input_chars,
                widgets::sys_text_input_deletions,
            )
            .run_if(in_state(GameStates::MainMenu))
        );
        app.add_systems(
            Update,
            (
                game_view::sys_add_casting_ui,
                game_view::sys_render_casters_ui,
            )
            .run_if(in_state(GameStates::Game))
        );
    }
}
