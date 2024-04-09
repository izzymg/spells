use bevy::{log, prelude::*};

use crate::ui::styles;
use crate::{GameState, GameStates};

/// Marker
#[derive(Component)]
pub(super) struct MenuItem;

/// Create menu items when our game state changes.
pub(super) fn sys_build_menus(mut commands: Commands, game_state: Res<GameState>) {
    if !(game_state.is_changed() && game_state.0 == GameStates::Menu) {
        return;
    }
    log::info!("spawning menu");
    commands.spawn((Camera2dBundle::default(), MenuItem));
    commands
        .spawn((
            MenuItem,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::BLACK),
                ..default()
            },
        ))
        .with_children(|layout| {
            layout.spawn((styles::title_text("SPELLS".into()), Label));
            layout.spawn(styles::btn()).with_children(|btn| {
                btn.spawn(styles::text("CONNECT".into()));
            });
        });
}

/// Despawn menu items when our game state changes.
pub(super) fn sys_cleanup_menu_items(
    mut commands: Commands,
    query: Query<Entity, With<MenuItem>>,
    game_state: Res<GameState>,
) {
    if !(game_state.is_changed() && game_state.0 != GameStates::Menu) {
        return;
    }
    log::info!("cleaning up menu");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
