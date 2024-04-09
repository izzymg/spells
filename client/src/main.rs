mod world_connection;
mod world_state;

use std::error::Error;

use bevy::{prelude::*, log};
use world_connection::WorldConnectionPlugin;
use world_state::WorldStatePlugin;

fn frame_sys() {
    log::debug!("beep");
}

fn main() -> Result<(), Box<dyn Error>> {
    {
        bevy::app::App::new()
            .add_systems(Update, frame_sys)
            .add_plugins((DefaultPlugins, WorldConnectionPlugin, WorldStatePlugin))
            .run();
        Ok(())
    }
}
