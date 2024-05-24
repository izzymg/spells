use bevy::prelude::*;
mod dev_follow_cam;
mod dev_replication;
mod dev_game_ui;

pub enum Scene {
    FollowCamera,
    Replication,
    GameUI,
}

pub struct DevScenesPlugin {
    pub scene: Scene,
}

impl Plugin for DevScenesPlugin {
    fn build(&self, app: &mut App) {
        match self.scene {
            Scene::GameUI => {
                app.add_plugins(dev_game_ui::DevGameUIPlugin);
            }
            Scene::FollowCamera => {
                app.add_plugins(dev_follow_cam::FollowCamDevScenePlugin);
            }
            Scene::Replication => {
                app.add_plugins(dev_replication::ReplicationDevScenePlugin);
            }
        }
    }
}
