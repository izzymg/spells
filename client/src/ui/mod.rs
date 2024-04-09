mod main_menu_control;
mod main_menu_view;
mod widgets;

use bevy::prelude::*;

use crate::GameStates;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                main_menu_view::sys_build_menus,
                main_menu_view::sys_cleanup_menu_items,
            ),
        );
        app.add_systems(
            Update,
            (
                main_menu_view::sys_on_click_connect_btn,
                main_menu_view::sys_update_status_text,
                main_menu_control::sys_menu_connect_ev,
                main_menu_control::sys_update_connection_status,
            ).in_set(GameStates::Menu),
        );
        app.insert_resource(main_menu_control::ConnectionStatus::default());
        app.add_event::<main_menu_control::ConnectEvent>();
        app.add_systems(
            Update,
            (
                widgets::sys_button_interaction,
                widgets::sys_text_input_chars,
                widgets::sys_text_input_deletions,
            )
                .in_set(GameStates::Menu),
        );
    }
}
