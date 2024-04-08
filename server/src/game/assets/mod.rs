mod auras;
mod spells;
use bevy::app::Plugin;

pub use crate::game::assets::auras::*;
pub use crate::game::assets::spells::*;

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(spells::get_spell_list_resource());
        app.insert_resource(auras::get_auras_resource());
    }
}
