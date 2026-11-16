use std::collections::VecDeque;
use avian3d::prelude::Position;
use bevy::{platform::collections::HashSet, prelude::*};
use lightyear::prelude::{ControlledBy, ReplicationSender, VisibilityExt};
use rand::RngExt;

use crate::{LoadingState, block_id_palette, blockdefs::BlockRegistry, voxels::{self, world_to_chunk}};
use crate::voxels::{ChunkPos, CHUNK_SIZE, chunk_to_world};
use crate::voxels::protocol::Chunk;
use crate::voxels::server::spawn_chunk;
use crate::voxels::shared::ChunkMap;

pub struct ServerChunksPlugin;

impl Plugin for ServerChunksPlugin {
	fn build(&self, app: &mut App) {
        app
            .add_systems(Update, 
                update_loaded_chunks
                    .before(voxels::client::generate_chunk_mesh)
                    .before(voxels::collision::shared::generate_chunk_collider)
                    .run_if(in_state(LoadingState::Loaded)
            ));

        app.add_systems(Update, (update_player_pending_chunks, update_visible_chunks).chain());
        app.add_observer(hide_chunks_for_new_client);
	}
}

#[derive(Component, Default)]
pub struct ChunkStream {
    center: Option<ChunkPos>,
    pending: VecDeque<ChunkPos>,
    visible: HashSet<ChunkPos>,
    hiding: HashSet<ChunkPos>,
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

fn update_loaded_chunks(
    block_registry: Res<BlockRegistry>,
    mut loaded: ResMut<ChunkMap>,
    mut loaders: Query<(&Position, &mut ChunkLoader)>,
    mut clients: Query<Entity, With<ReplicationSender>>,
    mut commands: Commands,
) {
    let mut desired: HashSet<IVec3> = HashSet::new();

    for (position, mut loader) in loaders.iter_mut() {
        let current_chunk = world_to_chunk(position.0);

        loader.last_chunk_pos = Some(current_chunk);

        desired.extend(chunks_in_range(current_chunk, loader.range));
    }

    block_id_palette!(block_registry, {
		"rg_air" => air_id,
        "rg_dirt" => dirt_id,
	});

    for &pos in &desired {
        if !loaded.chunks.contains_key(&pos) {
            let mut rng = rand::rng();

            let mut new_chunk = Chunk::from_block(air_id, pos);

            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let world_y = y as i32 + (pos.y * voxels::CHUNK_SIZE as i32);

                        if world_y >= 0 { continue; }

                        if rng.random_bool(0.3) {
                            new_chunk.set_voxel_xyz(x,y,z, dirt_id);
                        }
                    }
                }
            }

            spawn_chunk(new_chunk, clients, &mut commands);
        }
    }

    loaded.chunks.retain(|pos, chunk_entity| {
        if !desired.contains(pos) {
            commands.entity(*chunk_entity).despawn();
            false
        } else {
            true
        }
    });
}

pub const CHUNK_STREAM_RANGE: i32 = 5;

fn update_player_pending_chunks(
    mut players: Query<(&mut ChunkStream, &Position)>,
) {
    players.par_iter_mut().for_each(|(mut stream, pos)| {
        let old_center = stream.center;
        let new_center = world_to_chunk(pos.0);
        if old_center == Some(new_center) { return; }
        stream.center = Some(new_center);

        let mut wanted = chunks_in_range(new_center, CHUNK_STREAM_RANGE);
        wanted.retain(|c| { !stream.visible.contains(c) });

        let mut add: Vec<_> = wanted.iter().copied().collect();
        add.sort_by_key(|c| { (c - new_center).length_squared() });

        stream.pending = add.into();

        let keep = chunks_in_range(new_center, CHUNK_STREAM_RANGE + 1);
        let to_hide: HashSet<_> = stream.visible.difference(&keep).cloned().collect();

        stream.hiding = to_hide;
    });
}

const SIMULTANEOUS_VISIBILITY_CHANGE: i32 = 10;

fn update_visible_chunks(
    chunk_map: Res<ChunkMap>,
    mut players: Query<(&ControlledBy, &mut ChunkStream)>,
    mut commands: Commands,
) {
    for (controlled_by, mut stream) in &mut players {
        for _ in 0..SIMULTANEOUS_VISIBILITY_CHANGE {
            let Some(pos) = stream.pending.pop_front() else { break };

            let Some(chunk) = chunk_map.chunks.get(&pos) else {
                stream.pending.push_back(pos); // It might just not be loaded yet, put it back in the queue
                continue;
            };

            commands.gain_visibility(*chunk, controlled_by.owner);
            stream.visible.insert(pos);
        }

        //idk man
        let stream = &mut *stream;

        for &pos in stream.hiding.iter() {
            stream.visible.remove(&pos);

            let Some(chunk) = chunk_map.chunks.get(&pos) else { continue };

            commands.lose_visibility(*chunk, controlled_by.owner);
        }

        stream.hiding.clear();
    }
}

fn hide_chunks_for_new_client(
    trigger: On<Add<ReplicationSender>>,
    chunks: Query<Entity, With<Chunk>>,
    mut commands: Commands,
) {
    for chunk in &chunks {
        commands.lose_visibility(chunk, trigger.entity);
    }
}

fn chunks_in_range(position: ChunkPos, range: i32) -> HashSet<ChunkPos> {
    let mut desired: HashSet<IVec3> = HashSet::new();

    let range_sq = range * range;

    for x in -range..=range {
        for y in -range..=range {
            for z in -range..=range {
                if (ChunkPos::new(x,y,z) - position).length_squared() > range_sq { continue };

                desired.insert(position + IVec3::new(x, y, z));
            }
        }
    }

    desired
}