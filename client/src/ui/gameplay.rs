use crate::{input, ui::widgets};
use bevy::{log, prelude::*};
use lib_spells::shared;

const NAME_UI_GAP: f32 = 0.2;

#[derive(Component)]
pub struct GameplayUIWidget;
#[derive(Component, Debug)]
pub struct CastingSpellText(Entity);

#[derive(Component, Debug)]
pub struct PlayerUnitFrame;
#[derive(Component, Debug)]
pub struct TargetUnitFrame;
#[derive(Component, Debug)]
pub struct UnitFrameHealthText;
#[derive(Component, Debug)]
pub struct UnitFrameNameText;
#[derive(Component, Debug)]
pub struct UnitFrame;

#[derive(Default, Debug)]
pub struct TabTargetIndex(Option<usize>);

#[derive(Component, Debug, Default)]
pub struct UITarget;

pub fn sys_tab_target(
    mut commands: Commands,
    buttons: Res<input::ActionButtons>,
    are_targettable: Query<Entity, With<shared::Name>>,
    mut tab_target_index: Local<TabTargetIndex>,
    is_target: Query<Entity, With<UITarget>>,
) {
    if buttons.get_button_state(input::Action::Target) != input::ButtonState::Pressed {
        return;
    }

    // get a sorted list of every targetable entity
    let mut entity_list = are_targettable.iter().collect::<Vec<Entity>>();
    if entity_list.is_empty() {
        tab_target_index.0 = None;
        return;
    }
    entity_list.sort();

    // pick the next entity or wrap around to the start
    let next_index = match tab_target_index.0 {
        Some(i) => i + 1,
        None => 0,
    };

    let target = match entity_list.get(next_index) {
        Some(target) => {
            tab_target_index.0 = Some(next_index);
            *target
        }
        None => {
            tab_target_index.0 = Some(0);
            entity_list[0]
        }
    };

    // swap to new target
    if let Ok(current_target) = is_target.get_single() {
        commands.entity(current_target).remove::<UITarget>();
    }
    commands.entity(target).insert(UITarget);
}

fn unitframe(row: i16, col: i16) -> NodeBundle {
    let mut node = widgets::node();
    node.style = Style {
        grid_row: GridPlacement::start(row),
        grid_column: GridPlacement::start(col),
        ..Default::default()
    };
    node.background_color = BackgroundColor(Color::rgba(0.0, 1., 0., 0.2));
    node
}

pub fn sys_create_layout(mut commands: Commands) {
    commands.spawn((GameplayUIWidget, widgets::game_layout()));
}

pub fn sys_add_unitframes(
    mut commands: Commands,
    has_game_ui: Query<Entity, With<widgets::GameLayout>>,
) {
    let layout_entity = has_game_ui.single();
    let player_unitframe = commands
        .spawn((
            GameplayUIWidget,
            UnitFrame,
            PlayerUnitFrame,
            unitframe(4, 2),
        ))
        .id();
    let target_unitframe = commands
        .spawn((
            GameplayUIWidget,
            UnitFrame,
            TargetUnitFrame,
            unitframe(4, 5),
        ))
        .id();

    commands
        .entity(layout_entity)
        .insert_children(2, &[player_unitframe, target_unitframe]);
}

pub fn sys_setup_unitframes(
    mut commands: Commands,
    added_unitframe: Query<Entity, Added<UnitFrame>>,
) {
    for unitframe in added_unitframe.iter() {
        let name_text = commands
            .spawn((
                GameplayUIWidget,
                UnitFrameNameText,
                widgets::text("NONE".into()),
            ))
            .id();
        let hp_text = commands
            .spawn((
                GameplayUIWidget,
                UnitFrameHealthText,
                widgets::text("NONE".into()),
            ))
            .id();

        commands
            .entity(unitframe)
            .insert_children(2, &[name_text, hp_text]);
    }
}

pub fn sys_render_unitframe_health<F: Component, E: Component>(
    is_unitframe_children: Query<&Children, With<F>>,
    is_tracked_health: Query<&shared::Health, With<E>>,
    mut has_unitframe_health_text: Query<&mut Text, With<UnitFrameHealthText>>,
) {
    let unitframe_children = is_unitframe_children.single();
    let mut iter = has_unitframe_health_text.iter_many_mut(unitframe_children);
    let mut text = iter.fetch_next().unwrap();
    let hp = match is_tracked_health.get_single() {
        Ok(hp) => hp.0,
        Err(err) => match err {
            bevy::ecs::query::QuerySingleError::NoEntities(_) => {
                text.sections[0].value = "None".to_string();
                return;
            }
            bevy::ecs::query::QuerySingleError::MultipleEntities(_) => {
                panic!("attempted to render unitframe for query that returned multiple entities.");
            }
        },
    };
    text.sections[0].value = format!("{} HP", hp);
}

pub fn sys_render_unitframe_name<F: Component, E: Component>(
    is_unitframe_children: Query<&Children, With<F>>,
    is_tracked_name: Query<&shared::Name, With<E>>,
    mut has_unitframe_name_text: Query<&mut Text, With<UnitFrameNameText>>,
) {
    let unitframe_children = is_unitframe_children.single();
    let mut iter = has_unitframe_name_text.iter_many_mut(unitframe_children);
    let mut text = iter.fetch_next().unwrap();
    let name = match is_tracked_name.get_single() {
        Ok(name) => &name.0,
        Err(err) => match err {
            bevy::ecs::query::QuerySingleError::NoEntities(_) => {
                text.sections[0].value = "None".to_string();
                return;
            }
            bevy::ecs::query::QuerySingleError::MultipleEntities(_) => {
                panic!("attempted to render unitframe for query that returned multiple entities.");
            }
        },
    };
    text.sections[0].value = name.clone();
}
/// Add the child text entity & tag it when something is casting if there's no text already.
pub fn sys_add_casting_ui(
    mut commands: Commands,
    casting_added: Query<(Entity, Option<&Children>), Added<shared::CastingSpell>>,
    text_query: Query<Has<CastingSpellText>>,
) {
    for (caster_entity, caster_children) in casting_added.iter() {
        if let Some(children) = caster_children {
            let has = children
                .iter()
                .any(|c| text_query.get(*c).unwrap_or_default());
            if has {
                continue;
            }
        }

        commands.spawn((
            GameplayUIWidget,
            widgets::text("0".into()),
            CastingSpellText(caster_entity),
        ));
        commands.entity(caster_entity);
    }
}

/// Update casting spell text for casting parents. Despawn text with invalid entities.
pub fn sys_render_casters_ui(
    mut commands: Commands,
    is_casting: Query<&shared::CastingSpell>,
    mut has_casting_text: Query<(Entity, &CastingSpellText, &mut Text)>,
) {
    for (entity, casting_text, mut text) in has_casting_text.iter_mut() {
        if let Ok(casting_spell) = is_casting.get(casting_text.0) {
            text.sections[0].value = casting_spell.cast_timer.elapsed_secs().to_string();
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Component, Debug)]
pub struct AABB {
    half_extents: Vec3,
}

pub fn sys_add_aabb(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    mesh_changed: Query<(Entity, &Handle<Mesh>), Added<Handle<Mesh>>>,
) {
    for (entity, mesh) in mesh_changed.iter() {
        commands.entity(entity).insert(AABB {
            half_extents: meshes
                .get(mesh)
                .unwrap()
                .compute_aabb()
                .unwrap()
                .half_extents
                .into(),
        });
    }
}

#[derive(Component, Debug)]
pub struct NameUI {
    target: Entity,
}

pub fn sys_add_names_ui(
    mut commands: Commands,
    name_added: Query<(Entity, &shared::Name), Added<shared::Name>>,
) {
    for (entity, name) in name_added.iter() {
        commands.spawn((
            NameUI { target: entity },
            GameplayUIWidget,
            widgets::text(name.0.clone()).with_style(Style {
                display: Display::None,
                ..default()
            }),
        ));
    }
}

pub fn sys_render_names_ui(
    mut has_name_ui: Query<(&mut Style, &bevy::text::TextLayoutInfo, &NameUI)>,
    has_transform: Query<(&Transform, &AABB)>,
    is_camera: Query<(&GlobalTransform, &Camera)>,
) {
    let (camera_trans, camera) = is_camera.single();
    for (mut text_style, text_layout, name_ui) in has_name_ui.iter_mut() {
        if let Ok((target_transform, target_aabb)) = has_transform.get(name_ui.target) {
            if let Some(coords) = camera.world_to_viewport(
                camera_trans,
                target_transform.translation
                    + (Vec3::Y * (target_aabb.half_extents.y + NAME_UI_GAP)),
            ) {
                text_style.left = Val::Px(coords.x - (text_layout.logical_size.x / 2.));
                text_style.top = Val::Px(coords.y);
                text_style.display = Display::Flex;
            }
        }
    }
}

pub fn sys_clear_invalid_names_ui(mut commands: Commands, has_name_ui: Query<(Entity, &NameUI)>) {
    for (name_entity, name) in has_name_ui.iter() {
        if commands.get_entity(name.target).is_none() {
            commands.entity(name_entity).despawn_recursive();
        }
    }
}

pub fn sys_cleanup(
    has_multiplayer_ui: Query<Entity, With<GameplayUIWidget>>,
    mut commands: Commands,
) {
    for entity in has_multiplayer_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
