use crate::game;
use bevy::{log, prelude::*};

use super::widgets;

/// Tag
#[derive(Component, Debug)]
pub(super) struct CastingSpellText(Entity);

/// Add the child text entity & tag it when something is casting if there's no text already.
pub(super) fn sys_add_casting_ui(
    mut commands: Commands,
    caster_query: Query<(Entity, Option<&Children>), Added<game::CastingSpell>>,
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

        let spell_text = commands
            .spawn((
                widgets::title_text("BINGUS".into()),
                CastingSpellText(caster_entity),
            ))
            .id();
        commands.entity(caster_entity);
        log::debug!("spawned text");
    }
}

/// Update casting spell text for casting parents.
pub(super) fn sys_render_casters_ui(
    casting_query: Query<&game::CastingSpell>,
    mut text_query: Query<(&CastingSpellText, &mut Text)>,
) {
    for (casting_text, mut text) in text_query.iter_mut() {
        if let Ok(casting_spell) = casting_query.get(casting_text.0) {
            text.sections[0].value = casting_spell.timer.elapsed_secs().to_string();
        }
    }
}
