use bevy::prelude::*;
use rand::RngExt;

use crate::{GameState, blockdefs::BlockRegistry, voxels::{self, ChunkMaterialRes}};

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(GameState::Playing), spawn_chunks);
	}
}

fn spawn_chunks(
	chunk_material: Res<ChunkMaterialRes>,
    blocks: Res<BlockRegistry>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut rng = rand::rng();

	let mut debug_voxel_data = [[[0; 16]; 16]; 16];
    for (x, plane) in debug_voxel_data.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
        for (y, row) in plane.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
            for (z, voxel) in row.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
                *voxel = if (x & y & z) == 0 {
                    if rng.random_bool(0.5) {
                        blocks.get_id("rg_zart").unwrap()
                    } else {
                        blocks.get_id("rg_dirt").unwrap()
                    }
                } else { 
                    blocks.get_id("rg_air").unwrap()
                };
            }
        }
    }

    for x in 0usize..10 {
        for y in 0usize..10 {
            for z in 0usize..10 {
                commands.spawn((
                    voxels::Chunk {
                        coord: IVec3::new(x as i32, y as i32, z as i32),
                        voxels: debug_voxel_data,
                    },  
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    Transform::from_xyz(
                        (x * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE,
                        (y * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE,
                        (z * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE
                    ),
                    MeshMaterial3d(chunk_material.0.clone()),
                    voxels::DirtyChunk,
                ));
            }
        }
    }
}