use bevy::{log, prelude::*};

use crate::ui::widgets;


use super::main_menu_control::{self, ConnectEvent};

/// Marker
#[derive(Component)]
pub(super) struct MenuItem;

/// Marker
#[derive(Component)]
pub(super) struct ConnectButton;

/// Marker
#[derive(Component)]
pub(super) struct ConnectionStatusText;

/// Send connect event to controller
pub(super) fn sys_on_click_connect_btn(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ConnectButton>)>,
    addr_query: Query<&Text, With<widgets::TextInput>>,
    mut ev_w: EventWriter<main_menu_control::ConnectEvent>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            let text = &addr_query.single().sections[0].value;
            ev_w.send(ConnectEvent {
                address: text.clone(),
            });
        }
    }
}

pub(super) fn sys_update_status_text(
    status: Res<main_menu_control::ConnectionStatus>,
    mut query: Query<&mut Text, With<ConnectionStatusText>>,
) {
    if status.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            text.sections[0].value = status.status.clone();
        }
    }
}

pub(super) fn sys_create_main_menu(mut commands: Commands) {
    log::info!("spawning main menu");
    commands.spawn((Camera2dBundle::default(), MenuItem));
    let layout = commands
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
        .id();

    let status_text = commands
        .spawn((widgets::text("".into()), ConnectionStatusText))
        .id();
    let title = commands
        .spawn((widgets::title_text("SPELLS".into()), Label))
        .id();
    let connect_btn = commands
        .spawn((ConnectButton, widgets::btn()))
        .with_children(|btn| {
            btn.spawn(widgets::text("CONNECT".into()));
        })
        .id();
    commands
        .entity(layout)
        .push_children(&[status_text, title, connect_btn]);
    commands.add(widgets::CreateTextInputCommand {
        initial_val: "127.0.0.1:7776".into(),
        parent: Some(layout),
    });
}

pub(super) fn sys_cleanup_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MenuItem>>,
) {
    log::info!("cleaning up menu");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
