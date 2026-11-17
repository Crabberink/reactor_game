#![allow(dead_code)]
pub mod collision;
pub mod shared;
pub mod client;
pub mod protocol;
pub mod server;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::prelude::EntityCommandsDiffExt;

use crate::blockdefs::BlockId;
use crate::voxels::protocol::{Chunk, ChunkDif};
use crate::voxels::shared::ChunkMap;

/// Position of a block in the world
pub type BlockPos = IVec3;
/// Position of a chunk
pub type ChunkPos = IVec3;
/// Position of a block within its chunk
pub type LocalChunkPos = IVec3;

pub fn block_to_chunk_local(block_pos: BlockPos) -> LocalChunkPos {
	block_pos.rem_euclid(IVec3::splat(CHUNK_SIZE as i32))
}

pub fn block_to_chunk(block_pos: BlockPos) -> ChunkPos {
	block_pos.div_euclid(IVec3::splat(CHUNK_SIZE as i32))
}

pub fn world_to_block(world_pos: Vec3) -> BlockPos {
	world_pos.floor().as_ivec3()
}

pub fn world_to_chunk_local(world_pos: Vec3) -> LocalChunkPos {
	block_to_chunk_local(world_to_block(world_pos))
}

pub fn world_to_chunk(world_pos: Vec3) -> ChunkPos {
	block_to_chunk(world_to_block(world_pos))
}

pub fn block_to_world(block_pos: BlockPos) -> Vec3 {
	block_pos.as_vec3()
}

pub fn chunk_to_block(chunk_pos: ChunkPos) -> BlockPos {
	chunk_pos * CHUNK_SIZE as i32
}

pub fn chunk_to_world(chunk_pos: ChunkPos) -> Vec3 {
	block_to_world(chunk_to_block(chunk_pos))
}

pub const CHUNK_SIZE: usize = 16;
pub const VOXEL_SIZE: f32 = 1.0;

// YO there are genuine levels to this shit I barely know whats happening here
#[derive(SystemParam)]
pub struct VoxelWorld<'w, 's> {
	chunk_map: Res<'w, ChunkMap>,
	chunks: Query<'w, 's, &'static mut Chunk>,
	commands: Commands<'w, 's>
}

#[allow(dead_code)]
impl<'w, 's> VoxelWorld<'w, 's> {
	pub fn get_block(&self, block_pos: BlockPos) -> Option<BlockId> {
		let chunk_pos = block_to_chunk(block_pos);
		let chunk_local_pos = block_to_chunk_local(block_pos);

		let chunk_entity = self.chunk_map.chunks.get(&chunk_pos)?;
		let chunk = self.chunks.get(*chunk_entity).ok()?;

		Some(chunk.get_voxel(chunk_local_pos))
	}

	pub fn set_block(&mut self, block_pos: BlockPos, block_id: BlockId) -> bool {
		let chunk_pos = block_to_chunk(block_pos);
		let chunk_local_pos = block_to_chunk_local(block_pos);

		let Some(&chunk_entity) = self.chunk_map.chunks.get(&chunk_pos) else { return false; };
		if !self.chunks.contains(chunk_entity) { return false; }

		self.commands.entity(chunk_entity).apply_diff::<Chunk>(ChunkDif {
			id: block_id,
			pos: chunk_local_pos,
		});

		true
	}
}