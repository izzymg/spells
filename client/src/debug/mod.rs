use crate::{game::replication, world_connection};
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use iyes_perf_ui::prelude::*;

#[derive(Default)]
pub struct DebugPlugin;

#[derive(Component, Debug, Default)]
struct WorldLatencyUi;

impl PerfUiEntry for WorldLatencyUi {
    type Value = f64;
    type SystemParam = Option<SRes<world_connection::Connection>>;

    fn label(&self) -> &str {
        "World Latency (RTT)"
    }

    fn sort_key(&self) -> i32 {
        // on top
        -1
    }

    fn update_value(
        &self,
        conn: &mut <Self::SystemParam as SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        if let Some(conn) = conn {
            if let Some(latency) = conn.latency() {
                return Some(latency.as_secs_f64() * 1000.0);
            }
        }
        None
    }

    fn format_value(&self, value: &Self::Value) -> String {
        let mut s = iyes_perf_ui::utils::format_pretty_float(2, 3, *value);
        s.push_str(" ms");
        s
    }
}

/// Shows coordinates of the player
#[derive(Component, Debug, Default)]
struct PlayerLocationUi;

impl PerfUiEntry for PlayerLocationUi {
    type Value = Vec3;
    type SystemParam = SQuery<Read<Transform>, With<replication::PredictedPlayer>>;

    fn label(&self) -> &str {
        "Position"
    }

    fn sort_key(&self) -> i32 {
        // on top
        -1
    }

    fn format_value(&self, value: &Self::Value) -> String {
        format!("{}, {}, {}", value.x, value.y, value.z)
    }

    fn update_value(
        &self,
        transform: &mut <Self::SystemParam as SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        if let Ok(tr) = transform.get_single() {
            Some(tr.translation)
        } else {
            None
        }
    }
}
fn spawn_debug_ui(mut commands: Commands) {
    commands.spawn((
        iyes_perf_ui::PerfUiRoot::default(),
        PerfUiEntryFPS::default(),
        PerfUiEntryFrameTime::default(),
        PerfUiEntryEntityCount::default(),
        WorldLatencyUi,
        PlayerLocationUi,
    ));
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            iyes_perf_ui::PerfUiPlugin,
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        ));
        app.add_perf_ui_entry_type::<WorldLatencyUi>();
        app.add_perf_ui_entry_type::<PlayerLocationUi>();

        app.add_systems(Startup, spawn_debug_ui);
    }
}
