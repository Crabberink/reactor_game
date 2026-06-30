use bevy::{platform::collections::{HashMap, HashSet}, prelude::*};
use rand::RngExt;

use crate::{GameState, blockdefs::BlockRegistry, voxels::{self, ChunkMaterialRes, ChunkPos, VoxelMaterial, chunk_to_world, world_to_chunk}};

pub struct ChunksPlugin;

impl Plugin for ChunksPlugin {
	fn build(&self, app: &mut App) {
        app
            .init_resource::<LoadedChunks>()
            .add_systems(Update, update_loaded_chunks.run_if(in_state(GameState::Playing)));
		// app.add_systems(OnEnter(GameState::Playing), spawn_chunks);
	}
}

#[derive(Component)]
pub struct ChunkLoader {
    pub range: i32,
    pub last_chunk_pos: Option<IVec3>,
}

impl ChunkLoader {
    pub fn of_range(range: i32) -> ChunkLoader {
        ChunkLoader { range, last_chunk_pos: None }
    }
}

#[derive(Resource, Default)]
pub struct LoadedChunks {
    pub chunks: HashMap<IVec3, Entity>,
}

fn update_loaded_chunks(
    chunk_material: Res<ChunkMaterialRes>,
    mut loaded: ResMut<LoadedChunks>,
    mut loaders: Query<(&Transform, &mut ChunkLoader)>,
    mut commands: Commands,
) {
    let mut desired: HashSet<IVec3> = HashSet::new();

    for (transform, mut loader) in loaders.iter_mut() {
        let current_chunk = world_to_chunk(transform.translation);

        // if loader.last_chunk_pos == Some(current_chunk) {
        //     continue;
        // }
        loader.last_chunk_pos = Some(current_chunk);

        for x in -loader.range..=loader.range {
            for y in -loader.range..=loader.range {
                for z in -loader.range..=loader.range {
                    desired.insert(current_chunk + IVec3::new(x, y, z));
                }
            }
        }
    }

    for &pos in &desired {
        if !loaded.chunks.contains_key(&pos) {
            info!("Loaded chunk at pos {}", pos);
            let new_chunk = spawn_chunk(pos, &mut commands, chunk_material.0.clone());
            loaded.chunks.insert(pos, new_chunk);
        }
    }

    loaded.chunks.retain(|pos, chunk_entity| {
        if !desired.contains(pos) {
            warn!("Supposed to despawn chunk at pos {}", pos);
            commands.entity(*chunk_entity).despawn();
            false
        } else {
            true
        }
    });
}

fn spawn_chunk(chunk_pos: ChunkPos, commands: &mut Commands, material: Handle<VoxelMaterial>) -> Entity {
    let mut rng = rand::rng();

    let mut debug_voxel_data = [[[0; 16]; 16]; 16];
    for (x, plane) in debug_voxel_data.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
        for (y, row) in plane.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
            for (z, voxel) in row.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
                *voxel = if (x & y & z) == 0 {
                    if rng.random_bool(0.5) {
                        2
                    } else {
                        3
                    }
                } else { 
                    1
                };
            }
        }
    }

    commands.spawn((
        voxels::Chunk {
            coord: chunk_pos,
            voxels: debug_voxel_data,
        },
        Mesh3d::default(),
        Transform::from_translation(chunk_to_world(chunk_pos)),
        MeshMaterial3d(material),
        voxels::DirtyChunk,
    )).id()
}

// fn spawn_chunks(
// 	chunk_material: Res<ChunkMaterialRes>,
//     blocks: Res<BlockRegistry>,
// 	mut commands: Commands,
// 	mut meshes: ResMut<Assets<Mesh>>,
// ) {
//     let mut rng = rand::rng();

//     info!("Air id is {}", blocks.get_id("rg_air").unwrap());

// 	let mut debug_voxel_data = [[[0; 16]; 16]; 16];
//     for (x, plane) in debug_voxel_data.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
//         for (y, row) in plane.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
//             for (z, voxel) in row.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
//                 *voxel = if (x & y & z) == 0 {
//                     if rng.random_bool(0.5) {
//                         blocks.get_id("rg_zart").unwrap()
//                     } else {
//                         blocks.get_id("rg_dirt").unwrap()
//                     }
//                 } else { 
//                     blocks.get_id("rg_air").unwrap()
//                 };
//             }
//         }
//     }

//     for x in 0usize..10 {
//         for y in 0usize..10 {
//             for z in 0usize..10 {
//                 commands.spawn((
//                     voxels::Chunk {
//                         coord: IVec3::new(x as i32, y as i32, z as i32),
//                         voxels: debug_voxel_data,
//                     },  
//                     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
//                     Transform::from_xyz(
//                         (x * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE,
//                         (y * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE,
//                         (z * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE
//                     ),
//                     MeshMaterial3d(chunk_material.0.clone()),
//                     voxels::DirtyChunk,
//                 ));
//             }
//         }
//     }
// }