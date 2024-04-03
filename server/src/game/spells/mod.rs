pub mod casting;
pub mod resource;
pub mod spell_application;

use std::fmt;

use bevy::{
    app::{FixedUpdate, Plugin},
    ecs::schedule::IntoSystemConfigs,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellID(usize);

impl SpellID {
    pub fn get(self) -> usize {
        self.0
    }
}

impl From<usize> for SpellID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl fmt::Display for SpellID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(SPELL:{})", self.0)
    }
}

pub struct SpellsPlugin;

pub use casting::StartCastingEvent;

impl Plugin for SpellsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(resource::get_spell_list_resource())
            .add_event::<casting::StartCastingEvent>()
            .add_event::<spell_application::SpellApplicationEvent>()
            .add_systems(
                FixedUpdate,
                (
                    casting::handle_start_casting_event_system,
                    casting::tick_cast_system,
                    casting::check_finished_casts_system,
                    spell_application::handle_spell_applications_system
                )
                    .chain(),
            );
    }
}
