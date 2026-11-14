use bevy::prelude::*;
use bevy_replicon::prelude::Diffable;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use crate::blockdefs::BlockId;
use crate::voxels::*;

pub struct VoxelsProtocolPlugin;

impl Plugin for VoxelsProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<Chunk>().replicate_diff();
    }
}

#[derive(Component, Serialize, Deserialize, Reflect)]
pub struct Chunk {
    // FIX LATER I PROMISE THIS FIELD MIGHT BE USEFUL
    #[allow(dead_code)]
    pub coord: ChunkPos,
    pub voxels: [[[BlockId; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub(super) fn get_voxel(&self, local_pos: LocalChunkPos) -> BlockId {
        self.voxels[local_pos.x as usize][local_pos.y as usize][local_pos.z as usize]
    }
    pub(super) fn set_voxel(&mut self, local_pos: LocalChunkPos, block_id: BlockId) {
        self.voxels[local_pos.x as usize][local_pos.y as usize][local_pos.z as usize] = block_id
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChunkDif {
    pub(crate) id: BlockId,
    pub(crate) pos: ChunkPos,
}

impl bevy_replicon::prelude::Diffable for Chunk {
    type Diff = ChunkDif;

    fn apply_diff(&mut self, diff: &Self::Diff) -> Result<()> {
        self.set_voxel(diff.pos, diff.id);
        Ok(())
    }
}
