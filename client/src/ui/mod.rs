pub mod widgets;

use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                widgets::sys_button_interaction,
                widgets::sys_text_input_chars,
                widgets::sys_text_input_deletions,
            )
        );
    }
}
