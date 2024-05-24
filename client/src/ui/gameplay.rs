use crate::ui::widgets;
use bevy::{log, prelude::*};
use lib_spells::shared;

const NAME_UI_GAP: f32 = 0.2;

/// Tag
#[derive(Component)]
pub struct MultiplayerUIWidget;

/// Tag
#[derive(Component, Debug)]
pub struct CastingSpellText(Entity);

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
            MultiplayerUIWidget,
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
            NameUI {
                target: entity,
            },
            MultiplayerUIWidget,
            widgets::text(name.0.clone()),
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
            }
        } 
    }
}

pub fn sys_cleanup(
    has_multiplayer_ui: Query<Entity, With<MultiplayerUIWidget>>,
    mut commands: Commands,
) {
    for entity in has_multiplayer_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
