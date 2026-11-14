use avian3d::prelude::*;
use bevy::app::App;
use bevy::log::info;
use bevy::prelude::{Commands, Entity, OnEnter, Plugin, Res, Transform};
use lightyear::prelude::{NetworkTarget, Replicate};
use rand::RngExt;
use crate::{block_id_palette, voxels, LoadingState};
use crate::blockdefs::{BlockId, BlockRegistry};
use crate::voxels::{ChunkPos, CHUNK_SIZE, Chunk, chunk_to_world, DirtyChunkMesh};
use crate::voxels::collision::shared::DirtyChunkCollider;
use crate::voxels::shared::ChunkMap;

pub struct ServerVoxelsPlugin;

impl Plugin for ServerVoxelsPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(OnEnter(LoadingState::Loaded), spawn_test_chunk);
    }
}

pub fn spawn_chunk(
    chunk_pos: ChunkPos,
    voxel_data: [[[BlockId; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    commands: &mut Commands
) -> Entity {
    commands.spawn((
        Chunk {
            coord: chunk_pos,
            voxels: voxel_data,
        },
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

    let mut debug_voxel_data = [[[0; 16]; 16]; 16];
    for (_x, plane) in debug_voxel_data.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
        for (y, row) in plane.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
            for (_z, voxel) in row.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
                let world_y = y as i32 - 32;

                *voxel = if world_y < 0 {
                    if rng.random_bool(0.3) {
                        dirt_id
                    } else {
                        air_id
                    }
                } else {
                    air_id
                };
            }
        }
    }

    spawn_chunk(ChunkPos::ZERO, debug_voxel_data, &mut commands);
}