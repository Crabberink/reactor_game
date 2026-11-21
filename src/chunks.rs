use crate::voxels::protocol::Chunk;
use crate::voxels::server::spawn_chunk;
use crate::voxels::shared::ChunkMap;
use crate::voxels::{CHUNK_SIZE, ChunkPos};
use crate::{LoadingState, block_id_palette, blockdefs::BlockRegistry, voxels::{self, world_to_chunk}};
use avian3d::prelude::Position;
use bevy::tasks::futures::check_ready;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::{platform::collections::HashSet, prelude::*};
use lightyear::prelude::{ControlledBy, ReplicationSender, VisibilityExt};
use std::collections::VecDeque;

pub struct ServerChunksPlugin;

impl Plugin for ServerChunksPlugin {
	fn build(&self, app: &mut App) {
        app
            .add_systems(Update,
                (update_loaded_chunks, collect_generated_chunks).chain()
                .before(voxels::mesh::queue_generate_chunk_mesh)
                .before(voxels::collision::shared::queue_generate_chunk_collider)
                .run_if(in_state(LoadingState::Loaded)
            ));

        app.add_systems(Update, (update_player_pending_chunks, update_visible_chunks).chain());
        app.add_observer(hide_chunks_for_new_client);

        app.init_resource::<InFlight>();
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

#[derive(Component)]
struct ChunkGenTask { pos: ChunkPos, task: Task<Chunk> }

#[derive(Resource, Default, Deref, DerefMut)]
struct InFlight(HashSet<ChunkPos>);

const MAX_IN_FLIGHT: usize = 32;

fn update_loaded_chunks(
    block_registry: Res<BlockRegistry>,
    mut loaded: ResMut<ChunkMap>,
    mut in_flight: ResMut<InFlight>,
    mut loaders: Query<(&Position, &mut ChunkLoader)>,
    mut streams: Query<&mut ChunkStream>,
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

    let pool = AsyncComputeTaskPool::get();

    let mut chunks_to_queue: Vec<ChunkPos> = desired.iter()
        .filter(|p| !loaded.chunks.contains_key(*p) && !in_flight.contains(*p))
        .copied()
        .collect();

    chunks_to_queue.sort_by_key(|p| {
        loaders.iter()
            .map(|(pos, _)| (*p - world_to_chunk(pos.0)).length_squared())
            .min()
            .unwrap_or(i32::MAX)
    });

    for &pos in chunks_to_queue.iter().take(MAX_IN_FLIGHT.saturating_sub(in_flight.len())) {
        let gen_task = pool.spawn(async move {
            let chunk = if pos.y * CHUNK_SIZE as i32 >= 0 {
                Chunk::from_block(air_id, pos)
            } else {
                Chunk::from_block(dirt_id, pos)
            };

            chunk
        });

        commands.spawn(ChunkGenTask { pos, task: gen_task });
        in_flight.insert(pos);
    }

    loaded.chunks.retain(|pos, chunk_entity| {
        if desired.contains(pos) { return true; }

        commands.entity(*chunk_entity).despawn();
        for mut s in &mut streams {
            s.visible.remove(pos);
        }

        false
    });
}

fn collect_generated_chunks(
    mut tasks: Query<(Entity, &mut ChunkGenTask)>,
    mut in_flight: ResMut<InFlight>,
    clients: Query<Entity, With<ReplicationSender>>,
    mut commands: Commands,
) {
    for (e, mut handle) in tasks.iter_mut().take(8) {
        let Some(chunk) = check_ready(&mut handle.task) else { continue };

        in_flight.remove(&handle.pos);

        spawn_chunk(chunk, clients, &mut commands);
        commands.entity(e).despawn();
    }
}

pub const CHUNK_STREAM_RANGE: i32 = 5;

fn update_player_pending_chunks(
    mut players: Query<(&mut ChunkStream, &ChunkLoader, &Position)>,
) {
    players.par_iter_mut().for_each(|(mut stream, loader, pos)| {
        let old_center = stream.center;
        let new_center = world_to_chunk(pos.0);
        if old_center == Some(new_center) { return; }
        stream.center = Some(new_center);

        let mut wanted = chunks_in_range(new_center, loader.range);
        wanted.retain(|c| { !stream.visible.contains(c) });

        let mut add: Vec<_> = wanted.iter().copied().collect();
        add.sort_by_key(|c| { (c - new_center).length_squared() });

        stream.pending = add.into();

        let keep = chunks_in_range(new_center, loader.range + 1);
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
                let offset = ChunkPos::new(x,y,z);

                if offset.length_squared() > range_sq { continue; }

                desired.insert(position + IVec3::new(x, y, z));
            }
        }
    }

    desired
}