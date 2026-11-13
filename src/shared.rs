use crate::network::protocol::ProtocolPlugin;
use crate::network::shared::SharedNetworkPlugin;
use crate::player::shared::SharedPlayerPlugin;
use avian3d::PhysicsPlugins;
use avian3d::prelude::*;
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use lightyear::avian3d::plugin::{AvianReplicationMode, LightyearAvianPlugin};

#[derive(Clone)]
pub struct SharedPlugin;
impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        let rollback_resources = false;

        app.add_plugins(ProtocolPlugin);

        app.add_plugins(LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Position { sync_to_transform: false },
            rollback_resources,
            ..default()
        });

        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                // disable the position<>transform sync plugins as it is handled by lightyear_avian
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                // disable Sleeping plugin as it can mess up physics rollbacks
                .disable::<IslandPlugin>()
                .disable::<IslandSleepingPlugin>(),
        );

        // app.init_state::<GameState>();

        // app
            // .add_plugins(BlocksPlugin)
            // .add_plugins(VoxelsPlugin)
            // .add_plugins(ChunksPlugin)
            // .add_plugins(FrameTimeDiagnosticsPlugin::default())

        app.add_plugins(SharedNetworkPlugin);
        app.add_plugins(SharedPlayerPlugin);
    }
}