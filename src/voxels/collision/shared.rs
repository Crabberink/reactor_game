use crate::blockdefs::SharedBlockRegistry;
use crate::chunks::ChunkLoader;
use crate::voxels::shared::ChunkMap;
use crate::voxels::{Chunk, world_to_chunk};
use crate::{LoadingState, voxels};
use avian3d::parry::math::IVector;
use avian3d::prelude::{Collider, Position, RigidBodyDisabled};
use bevy::prelude::*;
use bevy::tasks::futures::check_ready;
use bevy::tasks::{AsyncComputeTaskPool, Task};

pub struct SharedVoxelCollisionPlugin;

impl Plugin for SharedVoxelCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, queue_generate_chunk_collider.run_if(in_state(LoadingState::Loaded)));
        app.add_systems(FixedUpdate, collect_generated_chunk_colliders.run_if(in_state(LoadingState::Loaded)));
        app.add_systems(FixedPreUpdate, gate_bodies);
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct DirtyChunkCollider;

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub struct ColliderGenTask(Task<Option<Collider>>);

#[derive(Component)]
pub struct ColliderDone;

const MAX_QUEUE_PER_FRAME: usize = 8;

pub fn queue_generate_chunk_collider(
    shared_registry: Res<SharedBlockRegistry>,
    mut commands: Commands,
    loaders: Query<&Position, With<ChunkLoader>>,
    dirty_chunks: Query<(Entity, &Chunk), (With<DirtyChunkCollider>, Without<ColliderGenTask>)>,
) {
    let _s = info_span!("queue_generate_chunk_collider").entered();

    let pool = AsyncComputeTaskPool::get();

    let mut ungenerated_chunks: Vec<(Entity, &Chunk)> = dirty_chunks.iter().collect();
    ungenerated_chunks.sort_by_key(|(_, c)| {
        loaders.iter()
            .map(|pos| (c.coord - world_to_chunk(pos.0)).length_squared())
            .min()
            .unwrap_or(i32::MAX)
    });

    for (entity, chunk) in ungenerated_chunks.into_iter().take(MAX_QUEUE_PER_FRAME) {
        let chunk = chunk.clone();
        let registry_handle = shared_registry.clone();

        let collider_task = pool.spawn(async move {
            let _s = info_span!("gen_collider_task").entered();

            let mut solid_voxels: Vec<IVector> = {
                let _alloc_span = info_span!("allocate voxel array").entered();

                Vec::with_capacity(voxels::CHUNK_SIZE * voxels::CHUNK_SIZE * voxels::CHUNK_SIZE)
            };

            let _s = info_span!("populate voxel array").entered();
            for x in 0..voxels::CHUNK_SIZE {
                for y in 0..voxels::CHUNK_SIZE {
                    for z in 0..voxels::CHUNK_SIZE {
                        let block_id = chunk.get_voxel_xyz(x,y,z);

                        let Some(def) = registry_handle.get_def(block_id) else {
                            warn!("Treating nonexistent block id: {} as non solid!", block_id);
                            continue;
                        };

                        if !def.solid { continue; }

                        solid_voxels.push(IVector::new(x as i32, y as i32, z as i32));
                    }
                }
            }

            if solid_voxels.is_empty() { return None; }

            let _s = info_span!("Create voxel component").entered();

            Some(Collider::voxels(Vec3::splat(voxels::VOXEL_SIZE), solid_voxels.as_slice()))
        });

        commands.entity(entity)
            .remove::<DirtyChunkCollider>()
            .insert(ColliderGenTask(collider_task));
    }
}

const MAX_COLLECT_PER_UPDATE: usize = 1;
pub fn collect_generated_chunk_colliders(
    mut tasks: Query<(Entity, &mut ColliderGenTask)>,
    mut commands: Commands,
) {
    let _s = info_span!("collect_generated_chunk_colliders").entered();

    let mut collected = 0;

    for (entity, mut task) in &mut tasks {
        let _s = info_span!("collect single collider").entered();
        if collected > MAX_COLLECT_PER_UPDATE { break; }

        let Some(optional_collider) = check_ready(&mut task.0) else { continue };
        collected += 1;

        commands.entity(entity)
            .try_insert(ColliderDone)
            .try_remove::<ColliderGenTask>();

        let Some(collider) = optional_collider else {
            commands.entity(entity).try_remove::<Collider>();
            continue;
        };


        commands.entity(entity)
            .try_insert(collider);
    }
}

/// Freezes bodies whose chunk doesn't yet have physics.
fn gate_bodies(
    map: Res<ChunkMap>,
    ready: Query<Entity, (With<Chunk>, With<ColliderDone>)>,
    mut bodies: Query<(Entity, &Position, Has<RigidBodyDisabled>)>,
    mut commands: Commands,
) {
    for (e, pos, disabled) in &mut bodies {
        let chunk_ok = map.chunks.get(&world_to_chunk(pos.0))
            .is_some_and(|&c| ready.contains(c));

        match (chunk_ok, disabled) {
            (false, false) => { commands.entity(e).insert(RigidBodyDisabled); }
            (true, true)   => { commands.entity(e).remove::<RigidBodyDisabled>(); }
            _ => {}
        }
    }
}