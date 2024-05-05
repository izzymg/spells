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
    type Value = u128;
    type SystemParam = Option<SRes<world_connection::Connection>>;

    fn label(&self) -> &str {
        "World ms (rtt)"
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
            if let Some(latency) = conn.get_latency() {
                return Some(latency.as_millis());
            }
        }
        None
    }
}

/// Shows coordinates of the player
#[derive(Component, Debug, Default)]
struct PlayerLocationUi;

impl PerfUiEntry for PlayerLocationUi {
    type Value = String;
    type SystemParam = SQuery<Read<Transform>, With<replication::ControlledPlayer>>;

    fn label(&self) -> &str {
        "Position"
    }

    fn sort_key(&self) -> i32 {
        // on top
        -1
    }

    fn update_value(
        &self,
        transform: &mut <Self::SystemParam as SystemParam>::Item<'_, '_>,
    ) -> Option<Self::Value> {
        if let Ok(tr) = transform.get_single() {
            Some(format!("{}, {}, {}", tr.translation.x, tr.translation.y, tr.translation.z))
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
