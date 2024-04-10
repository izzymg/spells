use crate::game;
use bevy::{log, prelude::*};

use super::widgets;

/// Tag
#[derive(Component, Debug, Default)]
pub struct CastingSpellText;

/// Add the child text entity & tag it when something is casting if there's no text already.
pub(super) fn sys_add_casting_ui(
    mut commands: Commands,
    caster_query: Query<(Entity, &Children), Added<game::CastingSpell>>,
    text_query: Query<Has<CastingSpellText>>,
) {
    for (caster_entity, caster_children) in caster_query.iter() {
        let has = caster_children
            .iter()
            .any(|c| text_query.get(*c).unwrap_or_default());
        if !has {
            let spell_text = commands
                .spawn((widgets::text("0".into()), CastingSpellText))
                .id();
            commands.entity(caster_entity).add_child(spell_text);
        }
    }
}

/// Update casting spell text for casting parents.
pub(super) fn sys_render_casters_ui(
    casting_query: Query<&game::CastingSpell>,
    mut text_query: Query<(&Parent, &mut Text), With<CastingSpellText>>,
) {
    for (text_parent, mut text) in text_query.iter_mut() {
        if let Ok(casting) = casting_query.get(text_parent.get()) {
            text.sections[0].value = casting.timer.elapsed_secs().to_string();
        }
    }
}
