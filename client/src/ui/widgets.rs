use bevy::{ecs::system::Command, input::keyboard, prelude::*};

pub(super) const ACCENT_COLOR: Color = Color::VIOLET;
pub const ACCENT_COLOR_DARK: Color = Color::PURPLE;
pub const BTN_BG: Color = Color::hsl(0.0, 0.0, 0.2);
pub const BTN_HOVER: Color = Color::hsl(0.0, 0.0, 0.1);

fn text_style() -> TextStyle {
    TextStyle {
        font: default(),
        font_size: 18.0,
        color: Color::hsla(0.0, 1.0, 1.0, 1.0),
    }
}

pub(super) fn btn() -> ButtonBundle {
    ButtonBundle {
        style: Style {
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::new(Val::Px(20.0), Val::Px(20.0), Val::Px(10.0), Val::Px(10.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        border_color: BorderColor(ACCENT_COLOR),
        background_color: BackgroundColor(BTN_BG),
        ..default()
    }
}

pub(super) fn text(value: String) -> TextBundle {
    TextBundle::from_section(value, text_style())
}

pub(super) fn title_text(value: String) -> TextBundle {
    TextBundle::from_section(
        value,
        TextStyle {
            font_size: 80.0,
            ..text_style()
        },
    )
}

/// Add styles to interactive buttons
pub(super) fn sys_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                background_color.0 = ACCENT_COLOR_DARK;
            }
            Interaction::Hovered => {
                background_color.0 = BTN_HOVER;
            }
            Interaction::None => {
                background_color.0 = BTN_BG;
            }
        }
    }
}

pub(super) fn sys_text_input_chars(
    mut events: EventReader<ReceivedCharacter>,
    mut edit_text: Query<&mut Text, With<TextInput>>,
) {
    for event in events.read() {
        if let Ok(mut text) = edit_text.get_single_mut() {
            text.sections[0].value.push_str(&event.char.to_string());
        }
    }
}

pub(super) fn sys_text_input_deletions(
    mut events: EventReader<keyboard::KeyboardInput>,
    mut edit_text: Query<&mut Text, With<TextInput>>,
) {
    for ev in events.read() {
        if ev.key_code == KeyCode::Backspace {
            let text = edit_text.single_mut();
            let mut chars = text.sections[0].value.chars();
            chars.next_back();
            edit_text.single_mut().sections[0].value = chars.as_str().into();
        }
    }
}

#[derive(Component, Debug)]
pub(super) struct TextInput;

pub(super) struct CreateTextInputCommand {
    pub initial_val: String,
    pub parent: Option<Entity>,
}

impl Command for CreateTextInputCommand {
    fn apply(self, world: &mut World) {
        let base = text(self.initial_val);
        let text = TextBundle {
            style: Style {
                border: UiRect::bottom(Val::Px(2.0)),
                ..base.style
            },
            ..base
        };
        let root = NodeBundle {
            border_color: BorderColor(ACCENT_COLOR),
            style: Style {
                border: UiRect::bottom(Val::Px(1.0)),
                ..Default::default()
            },
            ..default()
        };

        let text_entity = world.spawn((text, TextInput)).id();
        let root_entity = world.spawn(root).add_child(text_entity).id();
        if let Some(parent) = self.parent {
            world.entity_mut(parent).add_child(root_entity);
        }
    }
}
