pub mod collision;

use avian3d::dynamics::rigid_body::RigidBody;
use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashMap;
use bevy::render::render_resource::*;
use bevy::shader::ShaderRef;
use bevy::{asset::RenderAssetUsages, prelude::*};
use bevy::mesh::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};

use crate::GameState;
use crate::blockdefs::{BlockId, BlockRegistry};
use crate::voxels::collision::{DirtyChunkCollider, VoxelCollisionPlugin};

pub struct VoxelsPlugin;
impl Plugin for VoxelsPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<ChunkMap>();
		app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
		app.add_plugins(VoxelCollisionPlugin);
		app.add_systems(Update, generate_chunk_mesh.run_if(in_state(GameState::Playing)));
	}
}

pub type BlockPos = IVec3;
pub type ChunkPos = IVec3;
pub type LocalChunkPos = IVec3;

#[allow(dead_code)]
pub fn block_to_chunk_local(block_pos: BlockPos) -> LocalChunkPos {
	block_pos.rem_euclid(IVec3::splat(CHUNK_SIZE as i32))
}

#[allow(dead_code)]
pub fn block_to_chunk(block_pos: BlockPos) -> ChunkPos {
	block_pos.div_euclid(IVec3::splat(CHUNK_SIZE as i32))
}

#[allow(dead_code)]
pub fn world_to_block(world_pos: Vec3) -> BlockPos {
	world_pos.floor().as_ivec3()
}

#[allow(dead_code)]
pub fn world_to_chunk_local(world_pos: Vec3) -> LocalChunkPos {
	block_to_chunk_local(world_to_block(world_pos))
}

#[allow(dead_code)]
pub fn world_to_chunk(world_pos: Vec3) -> ChunkPos {
	block_to_chunk(world_to_block(world_pos))
}

#[allow(dead_code)]
pub fn block_to_world(block_pos: BlockPos) -> Vec3 {
	block_pos.as_vec3()
}

#[allow(dead_code)]
pub fn chunk_to_block(chunk_pos: ChunkPos) -> BlockPos {
	chunk_pos * CHUNK_SIZE as i32
}

#[allow(dead_code)]
pub fn chunk_to_world(chunk_pos: ChunkPos) -> Vec3 {
	block_to_world(chunk_to_block(chunk_pos))
}

pub const CHUNK_SIZE: usize = 16;
pub const VOXEL_SIZE: f32 = 1.0;

#[derive(Resource, Default)]
pub struct ChunkMap {
	pub chunks: HashMap<IVec3, Entity>,
}

#[derive(Component)]
pub struct Chunk {
	// FIX LATER I PROMISE THIS FIELD MIGHT BE USEFUL 
	#[allow(dead_code)]
	pub coord: ChunkPos,
	pub voxels: [[[BlockId; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
	fn get_voxel(&self, local_pos: LocalChunkPos) -> BlockId {
		self.voxels[local_pos.x as usize][local_pos.y as usize][local_pos.z as usize]
	}
	fn set_voxel(&mut self, local_pos: LocalChunkPos, block_id: BlockId) {
		self.voxels[local_pos.x as usize][local_pos.y as usize][local_pos.z as usize] = block_id
	}
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct DirtyChunkMesh;

// YO there are genuine levels to this shit I barely know whats happening here
#[derive(SystemParam)]
#[allow(dead_code)]
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
		let Ok(mut chunk) = self.chunks.get_mut(chunk_entity) else { return false; };

		chunk.set_voxel(chunk_local_pos, block_id);
		self.commands.entity(chunk_entity).insert((DirtyChunkMesh, DirtyChunkCollider));

		true
	}
}

pub fn spawn_chunk(chunk_pos: ChunkPos, voxel_data: [[[BlockId; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE], material: Handle<VoxelMaterial>, chunk_map: &mut ChunkMap, commands: &mut Commands) -> Entity {
	let chunk_entity = commands.spawn((
		Chunk {
			coord: chunk_pos,
            voxels: voxel_data,
        },
        Mesh3d::default(),
        Transform::from_translation(chunk_to_world(chunk_pos)),
        MeshMaterial3d(material),
        DirtyChunkMesh,
		RigidBody::Static,
        collision::DirtyChunkCollider,
    )).id();

	chunk_map.chunks.insert(chunk_pos, chunk_entity);

	chunk_entity
}

pub fn generate_chunk_mesh(
	registry: Res<BlockRegistry>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	dirty_chunks: Query<(Entity, &Chunk), With<DirtyChunkMesh>>,
	chunk_material: Res<ChunkMaterialRes>
) {
    for (entity, chunk) in dirty_chunks.iter() {
        let Some(new_mesh) = build_mesh(chunk, &registry) else {
			commands.entity(entity).remove::<Mesh3d>();
			continue;
		};


		commands.entity(entity).insert(Mesh3d(meshes.add(new_mesh)));
		// fucking annoying ass bevy quirk (this component change is required for the rendering system to pick up the entity)
		commands.entity(entity).insert( MeshMaterial3d(chunk_material.0.clone()));
		commands.entity(entity).remove::<DirtyChunkMesh>();
    }
}

fn is_solid(chunk: &Chunk, pos: IVec3, registry: &BlockRegistry) -> bool {
	if pos.x < 0 || pos.x >= CHUNK_SIZE as i32 { return false; }
	if pos.y < 0 || pos.y >= CHUNK_SIZE as i32 { return false; }
	if pos.z < 0 || pos.z >= CHUNK_SIZE as i32 { return false; }

	let id = chunk.voxels[pos.x as usize][pos.y as usize][pos.z as usize];
	let Some(def) = registry.get_def(id) else {
		warn!("Treating nonexistent block id: {} as transparent!", id);
		return false;
	};

	!(def.empty || def.transparent)
}

fn build_mesh(chunk: &Chunk, registry: &BlockRegistry) -> Option<Mesh> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);

	let mut positions: Vec<[f32; 3]> = Vec::new();
	let mut normals: Vec<[f32; 3]> = Vec::new();
	let mut uvs: Vec<[f32; 2]> = Vec::new();
	let mut indices: Vec<u32> = Vec::new();

	for x in 0..CHUNK_SIZE {
		for y in 0..CHUNK_SIZE {
			for z in 0..CHUNK_SIZE {
				let id = chunk.voxels[x][y][z];
				let Some(def) = registry.get_def(id) else { 
					warn!("Skipping nonexistent block id: {}!", id);
					continue;
				};
				
				if def.empty { continue; }

				if chunk.voxels[x][y][z] != 0 {
					let mut base = positions.len() as u32;

					let offset = Vec3::new(
						x as f32 * VOXEL_SIZE,
						y as f32 * VOXEL_SIZE,
						z as f32 * VOXEL_SIZE,
					);

					for face in VoxelFace::iter() {
						let ipos = IVec3::new(x as i32, y as i32, z as i32);
						let adj_pos: IVec3 = ipos + face.adj_offset();
						if is_solid(chunk, adj_pos, registry) { continue; }

						let face_data = generate_voxel_face(face, offset);

						let face_uvs = [
							[def.uv_rect.min.x, def.uv_rect.min.y],
							[def.uv_rect.max.x, def.uv_rect.min.y],
							[def.uv_rect.max.x, def.uv_rect.max.y],
							[def.uv_rect.min.x, def.uv_rect.max.y],
						];

						positions.extend_from_slice(&face_data.positions);
						normals.extend_from_slice(&face_data.normals);
						uvs.extend_from_slice(&face_uvs);
						indices.extend_from_slice(&[
							base, base + 2, base + 1,
							base, base + 3, base + 2,
						]);

						base += 4;
					}
				}
			}
		}
	}

	if indices.is_empty() { return None; }

	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
	mesh.insert_indices(Indices::U32(indices));

	Some(mesh)
}


#[derive(EnumIter)]
#[repr(u8)]
enum VoxelFace {
	Front,
	Right,
	Back,
	Left,
	Top,
	Bottom,
}

impl VoxelFace {
	fn adj_offset(&self) -> IVec3 {
		match self {
			VoxelFace::Front => IVec3::new(0, 0, -1),
			VoxelFace::Right => IVec3::new(1, 0, 0),
			VoxelFace::Back => IVec3::new(0, 0, 1),
			VoxelFace::Left => IVec3::new(-1, 0, 0),
			VoxelFace::Top => IVec3::new(0, 1, 0),
			VoxelFace::Bottom => IVec3::new(0, -1, 0),
		}
	}
}

// Holds a slice of the mesh data for a single face of a voxel, used for mesh generation
struct VoxelFaceData {
	positions: [[f32; 3]; 4],
	normals: [[f32; 3]; 4],
}

fn generate_voxel_face(face: VoxelFace, offset: Vec3) -> VoxelFaceData {

	let positions: [[f32; 3]; 4];
	let normals: [[f32; 3]; 4];

	match face {
		VoxelFace::Front => {
			positions = [
				[offset.x, offset.y, offset.z],
				[offset.x + VOXEL_SIZE, offset.y, offset.z],
				[offset.x + VOXEL_SIZE, offset.y + VOXEL_SIZE, offset.z],
				[offset.x, offset.y + VOXEL_SIZE, offset.z],
			];
			normals = [[0.0, 0.0, -1.0]; 4];
		}
		VoxelFace::Right => {
			positions = [
				[offset.x + VOXEL_SIZE, offset.y, offset.z],
				[offset.x + VOXEL_SIZE, offset.y, offset.z + VOXEL_SIZE],
				[offset.x + VOXEL_SIZE, offset.y + VOXEL_SIZE, offset.z + VOXEL_SIZE],
				[offset.x + VOXEL_SIZE, offset.y + VOXEL_SIZE, offset.z],
			];
			normals = [[1.0, 0.0, 0.0]; 4];
		}
		VoxelFace::Back => {
			positions = [
				[offset.x + VOXEL_SIZE, offset.y, offset.z + VOXEL_SIZE],
				[offset.x, offset.y, offset.z + VOXEL_SIZE],
				[offset.x, offset.y + VOXEL_SIZE, offset.z + VOXEL_SIZE],
				[offset.x + VOXEL_SIZE, offset.y + VOXEL_SIZE, offset.z + VOXEL_SIZE],
			];
			normals = [[0.0, 0.0, 1.0]; 4];
		}
		VoxelFace::Left => {
			positions = [
				[offset.x, offset.y, offset.z + VOXEL_SIZE],
				[offset.x, offset.y, offset.z],
				[offset.x, offset.y + VOXEL_SIZE, offset.z],
				[offset.x, offset.y + VOXEL_SIZE, offset.z + VOXEL_SIZE],
			];
			normals = [[-1.0, 0.0, 0.0]; 4];
		}
		VoxelFace::Top => {
			positions = [
				[offset.x, offset.y + VOXEL_SIZE, offset.z],
				[offset.x + VOXEL_SIZE, offset.y + VOXEL_SIZE, offset.z],
				[offset.x + VOXEL_SIZE, offset.y + VOXEL_SIZE, offset.z + VOXEL_SIZE],
				[offset.x, offset.y + VOXEL_SIZE, offset.z + VOXEL_SIZE],
			];
			normals = [[0.0, 1.0, 0.0]; 4];
		}
		VoxelFace::Bottom => {
			positions = [
				[offset.x, offset.y, offset.z + VOXEL_SIZE],
				[offset.x + VOXEL_SIZE, offset.y, offset.z + VOXEL_SIZE],
				[offset.x + VOXEL_SIZE, offset.y, offset.z],
				[offset.x, offset.y, offset.z],
			];
			normals = [[0.0, -1.0, 0.0]; 4];
		}
	}

	// What should I do with all the time I've saved not writing return
	VoxelFaceData {
		positions,
		normals,
	}
}

#[derive(Resource)]
pub struct ChunkMaterialRes(pub Handle<VoxelMaterial>);

#[derive(Asset, AsBindGroup, Reflect, Clone)]
pub struct VoxelMaterialExtension {
	#[texture(100, dimension = "2d")]
	#[sampler(101)]
	pub atlas: Handle<Image>,
}

impl MaterialExtension for VoxelMaterialExtension {
	fn fragment_shader() -> ShaderRef {
		"shaders/voxel.wgsl".into()
	}
}

pub type VoxelMaterial = ExtendedMaterial<StandardMaterial, VoxelMaterialExtension>;