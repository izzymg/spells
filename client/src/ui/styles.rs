use bevy::prelude::*;

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
