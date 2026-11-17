use crate::blockdefs::BlockRegistry;
use crate::voxels::{world_to_chunk, Chunk};
use crate::{LoadingState, voxels};
use avian3d::parry::math::IVector;
use avian3d::prelude::{Collider, Position, RigidBodyDisabled};
use bevy::prelude::*;
use crate::voxels::shared::ChunkMap;

pub struct SharedVoxelCollisionPlugin;

impl Plugin for SharedVoxelCollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, generate_chunk_collider.run_if(in_state(LoadingState::Loaded)));
        app.add_systems(FixedPreUpdate, gate_bodies);
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
        let mut solid_voxels: Vec<IVector> = {
            let _alloc_span = info_span!("allocate voxel array").entered();

            Vec::with_capacity(voxels::CHUNK_SIZE * voxels::CHUNK_SIZE * voxels::CHUNK_SIZE)
        };


        {
            let _pop_span = info_span!("populate voxel array").entered();

            for x in 0..voxels::CHUNK_SIZE {
                for y in 0..voxels::CHUNK_SIZE {
                    for z in 0..voxels::CHUNK_SIZE {
                        let block_id = chunk.get_voxel_xyz(x,y,z);

                        let Some(def) = block_registry.get_def(block_id) else {
                            warn!("Treating nonexistent block id: {} as non solid!", block_id);
                            continue;
                        };

                        if !def.solid { continue; }

                        solid_voxels.push(IVector::new(x as i32, y as i32, z as i32));
                    }
                }
            }
        }


        if solid_voxels.is_empty() {
            commands.entity(entity).remove::<Collider>();
        } else {
            let _collider_span = info_span!("insert collider").entered();
            commands.entity(entity).insert(Collider::voxels(Vec3::splat(voxels::VOXEL_SIZE), solid_voxels.as_slice()));
        }

        commands.entity(entity).remove::<DirtyChunkCollider>();
    }
}

/// Freezes bodies whose chunk doesn't yet have physics.
fn gate_bodies(
    map: Res<ChunkMap>,
    ready: Query<(), Without<DirtyChunkCollider>>,
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