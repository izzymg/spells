mod gameplay;
pub mod widgets;

use crate::{replication, window};
use bevy::prelude::*;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(window::WindowContext::Play),
            (
                gameplay::sys_create_layout,
                gameplay::sys_add_unitframes,
                gameplay::sys_setup_unitframes,
            )
                .chain(),
        );
        app.add_systems(
            Update,
            (
                (
                    // target > unitframe rendering
                    gameplay::sys_tab_target,
                    (
                        // player unitframe
                        gameplay::sys_render_unitframe_health::<
                            gameplay::PlayerUnitFrame,
                            replication::PredictedPlayer,
                        >,
                        gameplay::sys_render_unitframe_name::<
                            gameplay::PlayerUnitFrame,
                            replication::PredictedPlayer,
                        >,
                        // target unitframe
                        gameplay::sys_render_unitframe_name::<
                            gameplay::TargetUnitFrame,
                            gameplay::UITarget,
                        >,
                        gameplay::sys_render_unitframe_health::<
                            gameplay::TargetUnitFrame,
                            gameplay::UITarget,
                        >,
                    ),
                )
                    .chain(),
                (
                    gameplay::sys_add_casting_ui,
                    gameplay::sys_render_casters_ui,
                )
                    .chain(),
                (
                    gameplay::sys_add_aabb,
                    gameplay::sys_render_names_ui,
                    gameplay::sys_add_names_ui,
                    gameplay::sys_clear_invalid_names_ui,
                )
                    .chain(),
            )
                .run_if(in_state(window::WindowContext::Play)),
        );

        app.add_systems(OnExit(window::WindowContext::Play), gameplay::sys_cleanup);

        app.add_systems(
            Update,
            (
                widgets::sys_button_interaction,
                widgets::sys_text_input_chars,
                widgets::sys_text_input_deletions,
            )
                .run_if(in_state(window::WindowContext::Menu)),
        );
    }
}
