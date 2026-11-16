use avian3d::prelude::*;
use bevy::app::App;
use bevy::log::info;
use bevy::prelude::{Commands, Entity, OnEnter, Plugin, Query, Res, Transform, With};
use lightyear::prelude::{NetworkTarget, Replicate, ReplicationSender, VisibilityExt};
use rand::RngExt;
use crate::{block_id_palette, voxels, LoadingState};
use crate::blockdefs::{BlockId, BlockRegistry};
use crate::voxels::{ChunkPos, CHUNK_SIZE, Chunk, chunk_to_world};
use crate::voxels::collision::shared::DirtyChunkCollider;
use crate::voxels::shared::ChunkMap;

pub struct ServerVoxelsPlugin;

impl Plugin for ServerVoxelsPlugin {
    fn build(&self, app: &mut App) {

    }
}

pub fn spawn_chunk(
    chunk: Chunk,
    clients: Query<Entity, With<ReplicationSender>>,
    commands: &mut Commands
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