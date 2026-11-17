use crate::chunks::ServerChunksPlugin;
use crate::network::server::ServerNetworkPlugin;
use crate::network::shared::SEND_INTERVAL;
use crate::player::server::ServerPlayerPlugin;
use crate::voxels::server::ServerVoxelsPlugin;
use bevy::app::{App, Plugin};
use lightyear::prelude::ReplicationMetadata;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ReplicationMetadata::new(SEND_INTERVAL));

        app.add_plugins(ServerNetworkPlugin);
        app.add_plugins(ServerPlayerPlugin);
        app.add_plugins(ServerVoxelsPlugin);
        app.add_plugins(ServerChunksPlugin);
    }
}