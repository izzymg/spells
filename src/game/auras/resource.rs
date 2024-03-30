use bevy::ecs::system::Resource;

use super::{AuraData, AuraType};

#[derive(Resource)]
pub(super) struct AuraList(pub Vec<AuraData>);

impl AuraList {
    pub(super) fn get_aura_data(&self, id: usize) -> Option<&AuraData> {
        self.0.get(id)
    }
}

pub(super) fn get_aura_list_resource() -> AuraList {
    AuraList(
            vec![
                AuraData::new("Burning".into(), 2000, AuraType::SHIELD),
                AuraData::new("Burning".into(), 50000, AuraType::SHIELD),
            ]
    )
}
