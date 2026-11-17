use crate::blockdefs::BlockId;
use crate::voxels::*;
use bevy::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

pub struct VoxelsProtocolPlugin;

impl Plugin for VoxelsProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<Chunk>().replicate_diff();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct PalettedData {
    types: Vec<BlockId>,
    log2_bits: u8,
    data: Vec<u64>
}

impl PalettedData {
    fn log2_width_for(palette_len: usize) -> u8 {
        let needed = (usize::BITS - (palette_len - 1).leading_zeros()).max(1);
        needed.next_power_of_two().trailing_zeros() as u8 // 1,2,4,8,16 -> 0..4
    }

    fn set_raw(&mut self, i: usize, raw: u64) {
        let bit = i << self.log2_bits;
        let width = 1usize << self.log2_bits;
        let mask = ((1u64 << width) - 1) << (bit & 63);
        let word = &mut self.data[bit >> 6];
        *word = (*word & !mask) | (raw << (bit & 63));
    }

    pub fn set(&mut self, i: usize, id: BlockId) {
        let raw = match self.types.iter().position(|&t| t == id) {
            Some(p) => p,
            None => {
                self.types.push(id);
                let new_log2 = Self::log2_width_for(self.types.len());
                if new_log2 != self.log2_bits {
                    self.repack(new_log2);
                }
                self.types.len() - 1
            }
        };
        self.set_raw(i, raw as u64);
    }


    fn raw_get(data: &[u64], log2: u8, i: usize) -> u64 {
        let bit = i << log2;
        let width = 1usize << log2;
        (data[bit >> 6] >> (bit & 63)) & ((1u64 << width) - 1)
    }

    fn raw_set(data: &mut [u64], log2: u8, i: usize, v: u64) {
        let bit = i << log2;
        let width = 1usize << log2;
        let shift = bit & 63;
        let mask = ((1u64 << width) - 1) << shift;
        let word = &mut data[bit >> 6];
        *word = (*word & !mask) | (v << shift);
    }
    fn words_for(log2: u8) -> usize {
        (CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE << log2).div_ceil(64)
    }
    fn repack(&mut self, new_log2: u8) {
        let mut new_data = vec![0u64; Self::words_for(new_log2)];
        for i in 0..(CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE) {
            Self::raw_set(&mut new_data, new_log2, i, Self::raw_get(&self.data, self.log2_bits, i));
        }
        self.data = new_data;
        self.log2_bits = new_log2;
    }

    pub fn new(first: BlockId) -> Self {
        Self { types: vec![first], log2_bits: 0, data: vec![0; Self::words_for(0)] }
    }

    pub fn get(&self, i: usize) -> BlockId {
        self.types[Self::raw_get(&self.data, self.log2_bits, i) as usize]
    }
}

#[derive(Component, Serialize, Deserialize, Reflect)]
pub struct Chunk {
    pub coord: ChunkPos,
    data: PalettedData,
}

const N: usize = CHUNK_SIZE;
impl Chunk {
    fn idx(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < N && y < N && z < N);
        (y * N + z) * N + x // y-major: each horizontal layer is contiguous
    }

    pub fn get_voxel(&self, local_pos: LocalChunkPos) -> BlockId {
        self.data.get(Self::idx(local_pos.x as usize, local_pos.y as usize, local_pos.z as usize))
    }

    pub fn set_voxel(&mut self, local_pos: LocalChunkPos, block_id: BlockId) {
        self.data.set(Self::idx(local_pos.x as usize, local_pos.y as usize, local_pos.z as usize), block_id)
    }

    pub fn get_voxel_xyz(&self, x: usize,  y: usize, z: usize) -> BlockId {
        self.data.get(Self::idx(x,y,z))
    }

    pub fn set_voxel_xyz(&mut self, x: usize,  y: usize, z: usize, block_id: BlockId) {
        self.data.set(Self::idx(x,y,z), block_id)
    }

    pub fn from_block(block: BlockId, pos: ChunkPos) -> Self {
        Chunk {
            coord: pos,
            data: PalettedData::new(block),
        }
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
