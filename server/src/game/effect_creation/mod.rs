mod spells;
mod auras;
use bevy::prelude::*;

use super::ServerSets;

pub struct EffectCreationPlugin;

impl Plugin for EffectCreationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            FixedUpdate,
            (
                spells::sys_validate_cast_targets,
                spells::sys_tick_casts,
                spells::sys_dispatch_finished_casts,
                spells::sys_remove_finished_casts,
                spells::sys_spell_application_ev,
            )
                .chain()
                .in_set(ServerSets::EffectCreation),
        );
        app.add_systems(FixedUpdate, (
            auras::sys_apply_aura_tick,
            auras::sys_tick_ticking_auras,
        ));
    }
}
