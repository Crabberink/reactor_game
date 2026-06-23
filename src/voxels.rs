use bevy::render::render_resource::*;
use bevy::shader::ShaderRef;
use bevy::{asset::RenderAssetUsages, prelude::*};
use bevy::mesh::*;
use strum::IntoEnumIterator;
use std::collections::HashMap;
use strum_macros::EnumIter;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};

use crate::GameState;
use crate::blockdefs::{BlockId, BlockRegistry};

#[derive(Component)]
pub struct Chunk {
	pub coord: IVec3,
	pub voxels: [[[BlockId; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct DirtyChunk;

//stop the unused warning for now, will be used later when we implement chunk loading and unloading
#[allow(dead_code)]
#[derive(Resource)]
pub struct ChunkMap(HashMap<IVec3, Entity>);

pub struct VoxelsPlugin;
impl Plugin for VoxelsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());
		app.insert_resource(ChunkMap(HashMap::new()));
		app.add_systems(Update, generate_chunk_mesh.run_if(in_state(GameState::Playing)));
	}
}

pub const CHUNK_SIZE: usize = 16;
pub const VOXEL_SIZE: f32 = 1.0;

fn generate_chunk_mesh(
	registry: Res<BlockRegistry>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut dirty_chunks: Query<(Entity, &mut Mesh3d, &Chunk), With<DirtyChunk>>
) {
    for (entity, mut mesh_handle, chunk) in dirty_chunks.iter_mut() {
        let new_mesh = build_mesh(chunk, &registry);
		
        mesh_handle.0 = meshes.add(new_mesh);

		commands.entity(entity).remove::<DirtyChunk>(); 
    }
}

fn is_solid(chunk: &Chunk, pos: IVec3, registry: &BlockRegistry) -> bool {
	if pos.x < 0 || pos.x >= CHUNK_SIZE as i32 { return false; }
	if pos.y < 0 || pos.y >= CHUNK_SIZE as i32 { return false; }
	if pos.z < 0 || pos.z >= CHUNK_SIZE as i32 { return false; }

	let id = chunk.voxels[pos.x as usize][pos.y as usize][pos.z as usize];
	let def = registry.get_def(id);

	!(def.empty || def.transparent)
}

fn build_mesh(chunk: &Chunk, registry: &BlockRegistry) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);

	let mut positions: Vec<[f32; 3]> = Vec::new();
	let mut normals: Vec<[f32; 3]> = Vec::new();
	let mut uvs: Vec<[f32; 2]> = Vec::new();
	let mut indices: Vec<u32> = Vec::new();

	for x in 0..CHUNK_SIZE {
		for y in 0..CHUNK_SIZE {
			for z in 0..CHUNK_SIZE {
				let id = chunk.voxels[x][y][z];
				let def = registry.get_def(id);
				
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

	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
	mesh.insert_indices(Indices::U32(indices));

	mesh
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