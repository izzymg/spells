use std::time::Duration;

pub struct SpellData {
    pub name: String,
    pub cast_time: Duration,
    pub target_health_effect: Option<i64>,
    pub self_health_effect: Option<i64>,
}

impl SpellData {
    pub fn new(name: String, cast_ms: u64) -> SpellData {
        SpellData {
            name: name,
            cast_time: Duration::from_millis(cast_ms),
            self_health_effect: None,
            target_health_effect: None,
        }
    }
    pub fn new_damage(name: String, cast_ms: u64, damage: i64) -> SpellData {
        SpellData {
            name: name,
            cast_time: Duration::from_millis(cast_ms),
            self_health_effect: None,
            target_health_effect: Some(damage),
        }
    }
}