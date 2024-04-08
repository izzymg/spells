mod spells;
use bevy::prelude::*;

use super::ServerSets;

pub struct EffectCreationPlugin;

impl Plugin for EffectCreationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            FixedUpdate,
            (
                spells::sys_start_casting_ev,
                spells::sys_tick_casts,
                spells::sys_validate_cast_targets,
                spells::sys_finish_casts,
                spells::sys_spell_application_ev,
            )
                .chain()
                .in_set(ServerSets::EffectCreation),
        );
    }
}
