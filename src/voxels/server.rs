use crate::voxels::Chunk;
use bevy::app::App;
use bevy::prelude::{Commands, Entity, Plugin, Query, With};
use lightyear::prelude::{NetworkTarget, Replicate, ReplicationSender, VisibilityExt};

pub struct ServerVoxelsPlugin;

impl Plugin for ServerVoxelsPlugin {
    fn build(&self, app: &mut App) {}
}

pub fn spawn_chunk(
    chunk: Chunk,
    clients: Query<Entity, With<ReplicationSender>>,
    commands: &mut Commands,
) -> Entity {
    let chunk = commands.spawn((
        chunk,
        // Everyone for now
        Replicate::to_clients(NetworkTarget::All),
    )).id();

    for entity in clients.iter() {
        commands.lose_visibility(chunk, entity);
    }

    chunk
}