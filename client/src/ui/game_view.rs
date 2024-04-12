use bevy::{log, prelude::*};

use super::widgets;

use lib_spells::shared;

/// Tag
#[derive(Component, Debug)]
pub(super) struct CastingSpellText(Entity);

/// Add the child text entity & tag it when something is casting if there's no text already.
pub(super) fn sys_add_casting_ui(
    mut commands: Commands,
    caster_query: Query<(Entity, Option<&Children>), Added<shared::CastingSpell>>,
    text_query: Query<Has<CastingSpellText>>,
) {
    for (caster_entity, caster_children) in caster_query.iter() {
        log::debug!("I LOVE CASTING SPELLS");
        if let Some(children) = caster_children {
            let has = children
                .iter()
                .any(|c| text_query.get(*c).unwrap_or_default());
            if has {
                continue;
            }
        }

        commands.spawn((widgets::text("0".into()), CastingSpellText(caster_entity)));
        commands.entity(caster_entity);
        log::debug!("spawned text");
    }
}

/// Update casting spell text for casting parents. Despawn text with invalid entities.
pub(super) fn sys_render_casters_ui(
    mut commands: Commands,
    casting_query: Query<&shared::CastingSpell>,
    mut text_query: Query<(Entity, &CastingSpellText, &mut Text)>,
) {
    for (entity, casting_text, mut text) in text_query.iter_mut() {
        if let Ok(casting_spell) = casting_query.get(casting_text.0) {
            text.sections[0].value = casting_spell.cast_timer.elapsed_secs().to_string();
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}
