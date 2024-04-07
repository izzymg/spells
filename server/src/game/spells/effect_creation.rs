use bevy::{
    ecs::{
        event::{EventReader, EventWriter},
        schedule::IntoSystemConfigs,
        system::Res,
    },
    log,
};

use crate::game::effects;

use super::{resource::SpellList, SpellApplicationEvent};

fn sys_spell_application_ev(
    spell_list: Res<SpellList>,
    mut effect_ev_w: EventWriter<effects::EffectQueueEvent>,
    mut spell_ev_r: EventReader<SpellApplicationEvent>,
) {
    for ev in spell_ev_r.read() {
        if let Some(spell_data) = spell_list.get_spell_data(ev.spell_id) {
            effect_ev_w.send(effects::EffectQueueEvent {
                target: ev.target,
                health_effect: spell_data.target_health_effect,
                aura_effect: spell_data.target_aura_effect,
            });
        } else {
            log::warn!("no spell {}", ev.spell_id);
        }
    }
}

pub fn get_configs() -> impl IntoSystemConfigs<()> {
    (sys_spell_application_ev,).into_configs()
}
