use avian3d::{parry::math::IVector, prelude::*};
use bevy::prelude::*;

use crate::{GameState, blockdefs::BlockRegistry, voxels::{self, Chunk}};

pub struct VoxelCollisionPlugin;
impl Plugin for VoxelCollisionPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(FixedUpdate, generate_chunk_collider.run_if(in_state(GameState::Playing)));
	}
}


#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct DirtyChunkCollider;

pub fn generate_chunk_collider(
	block_registry: Res<BlockRegistry>,
	mut commands: Commands,
	dirty_chunks: Query<(Entity, &Chunk), With<DirtyChunkCollider>>,
) {
	for (entity, chunk) in dirty_chunks.iter() {
		// This thing gonna be gone in a shit and a fart so who cares if we over allocate
		let mut solid_voxels: Vec<IVector> = Vec::with_capacity(voxels::CHUNK_SIZE * voxels::CHUNK_SIZE * voxels::CHUNK_SIZE);

		for x in 0..voxels::CHUNK_SIZE {
			for y in 0..voxels::CHUNK_SIZE {
				for z in 0..voxels::CHUNK_SIZE {
					let block_id = chunk.voxels[x][y][z];

					let Some(def) = block_registry.get_def(block_id) else {
						warn!("Treating nonexistent block id: {} as non solid!", block_id);
						continue;
					};

					if !def.solid { continue; }

					solid_voxels.push(IVector::new(x as i32, y as i32, z as i32));
				}
			}
		}

		if solid_voxels.is_empty() {
			commands.entity(entity).remove::<Collider>();
		} else {
			commands.entity(entity).insert(Collider::voxels(Vec3::splat(voxels::VOXEL_SIZE), solid_voxels.as_slice()));
		}

		commands.entity(entity).remove::<DirtyChunkCollider>();
	}
}