use avian3d::prelude::*;
use bevy::app::App;
use bevy::log::info;
use bevy::prelude::{Commands, Entity, OnEnter, Plugin, Res, Transform};
use lightyear::prelude::{NetworkTarget, Replicate};
use rand::RngExt;
use crate::{block_id_palette, voxels, LoadingState};
use crate::blockdefs::{BlockId, BlockRegistry};
use crate::voxels::{ChunkPos, CHUNK_SIZE, Chunk, chunk_to_world};
use crate::voxels::collision::shared::DirtyChunkCollider;
use crate::voxels::shared::ChunkMap;

pub struct ServerVoxelsPlugin;

impl Plugin for ServerVoxelsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(LoadingState::Loaded), spawn_test_chunk);
    }
}

pub fn spawn_chunk(
    chunk: Chunk,
    commands: &mut Commands
) -> Entity {
    commands.spawn((
        chunk,
        // Everyone for now
        Replicate::to_clients(NetworkTarget::All),
    )).id()
}

fn spawn_test_chunk(
    block_registry: Res<BlockRegistry>,
    mut commands: Commands,
) {
    block_id_palette!(block_registry, {
		"rg_air" => air_id,
        "rg_dirt" => dirt_id,
	});

    let mut rng = rand::rng();

    let mut new_chunk = Chunk::from_block(air_id, ChunkPos::ZERO);

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_y = y as i32 - 32;

                if world_y >= 0 { continue; }

                if rng.random_bool(0.3) {
                    new_chunk.set_voxel_xyz(x,y,z, dirt_id);
                }
            }
        }
    }

    spawn_chunk(new_chunk, &mut commands);
}