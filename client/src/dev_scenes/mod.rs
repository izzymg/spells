use bevy::prelude::*;
mod dev_follow_cam;
mod dev_replication;

pub enum Scene {
    FollowCamera,
    Replication,
}

pub struct DevScenesPlugin {
    pub scene: Scene,
}

impl Plugin for DevScenesPlugin {
    fn build(&self, app: &mut App) {
        match self.scene {
            Scene::FollowCamera => {
                app.add_plugins(dev_follow_cam::FollowCamDevScenePlugin);
            },
            Scene::Replication => {
                app.add_plugins(dev_replication::ReplicationDevScenePlugin);
            }
        }
    }
}
