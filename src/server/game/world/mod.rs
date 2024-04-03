/// snapshots of world
use bevy::{
    app::{FixedLast, Last, Plugin},
    ecs::system::{In, Query},
    prelude::IntoSystem,
    utils::dbg,
};

use crate::game::spells::casting;

#[derive(Debug)]
pub struct CasterState {
    pub timer: u128,
    pub max_timer: u128,
    pub spell_id: usize,
}

#[derive(Debug, Default)]
pub struct WorldState {
    pub casters: Vec<CasterState>,
}

fn create_state_sys() -> WorldState {
    WorldState::default()
}

fn state_casters_sys(
    In(mut world_state): In<WorldState>,
    query: Query<&casting::CastingSpell>,
) -> WorldState {
    world_state.casters = query
        .iter()
        .map(|caster| CasterState {
            max_timer: caster.cast_timer.duration().as_millis().into(),
            spell_id: caster.spell_id.get(),
            timer: caster.cast_timer.elapsed().as_millis().into(),
        })
        .collect();
    world_state
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(FixedLast, create_state_sys.pipe(state_casters_sys).map(dbg));
    }
}
