use std::time::Duration;

#[derive(Debug)]
pub struct Spell {
    pub name: String,
    pub hit_points: i64,
    pub cast_time: Duration,
}
