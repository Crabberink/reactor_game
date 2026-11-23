use crate::voxels::collision::shared::{DirtyChunkCollider, SharedVoxelCollisionPlugin, queue_generate_chunk_collider};
use crate::voxels::mesh::{queue_generate_chunk_mesh, ChunkNeighbor, DirtyChunkMesh};
use crate::voxels::{Chunk, chunk_to_world};
use avian3d::prelude::*;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use strum::IntoEnumIterator;

pub struct SharedVoxelsPlugin;

impl Plugin for SharedVoxelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkMap>();
        app.add_plugins(SharedVoxelCollisionPlugin);
        app.add_systems(Update, mark_changed_chunks.before(queue_generate_chunk_mesh).before(queue_generate_chunk_collider));
        app.add_observer(setup_added_chunk);
        app.add_observer(cleanup_removed_chunk);
    }
}

#[derive(Resource, Default)]
pub struct ChunkMap {
    pub chunks: HashMap<IVec3, Entity>,
}

fn setup_added_chunk(
    add: On<Add<Chunk>>,
    chunks: Query<&Chunk>,
    mut chunk_map: ResMut<ChunkMap>,
    mut commands: Commands,
) {
    let entity = add.entity;

    let Ok(chunk) = chunks.get(entity) else { return; };

    chunk_map.chunks.insert(chunk.coord, entity);
    commands.entity(entity).insert((
        Position(chunk_to_world(chunk.coord)),
        RigidBody::Static,
        DirtyChunkCollider,
        DirtyChunkMesh,
    ));
}

fn cleanup_removed_chunk(
    remove: On<Remove<Chunk>>,
    chunks: Query<&Chunk>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    let entity = remove.entity;
    let Ok(chunk) = chunks.get(entity) else { return; };

    // only remove the entry if it still points at THIS entity
    if chunk_map.chunks.get(&chunk.coord) == Some(&entity) {
        chunk_map.chunks.remove(&chunk.coord);
    }
}

fn mark_changed_chunks(
    chunk_map: Res<ChunkMap>,
    chunks: Query<(Entity, &Chunk), Changed<Chunk>>,
    mut commands: Commands,
) {
    for (entity, chunk) in &chunks {
        commands.entity(entity).try_insert((DirtyChunkMesh, DirtyChunkCollider));

        for n in ChunkNeighbor::iter() {
            if let Some(&e) = chunk_map.chunks.get(&(chunk.coord + n.adj_offset())) {
                commands.entity(e).try_insert(DirtyChunkMesh);
            }
        }
    }
}