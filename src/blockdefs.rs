use bevy::asset::io::Reader;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::asset::{Asset, AssetLoader, LoadContext, LoadedFolder};
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;
use bevy::pbr::ExtendedMaterial;

use crate::GameState;
use crate::voxels::{ChunkMaterialRes, VoxelMaterial, VoxelMaterialExtension};

pub struct BlocksPlugin;
impl Plugin for BlocksPlugin {
	fn build(&self, app: &mut App) {		
		app
			.init_resource::<BlockAssets>()
			.init_resource::<BlockRegistry>()
			.init_asset::<BlockDefinition>()
			.init_asset_loader::<BlockDefinitionAssetLoader>()
			.add_systems(Startup, load_blocks)
			.add_systems(Update, register_blocks.run_if(in_state(GameState::Loading)));
	}
}

#[allow(dead_code)]
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct BlockDefinition {
	// Defined values
	pub name: String,
	pub key: String,
	#[serde(default)]
	pub hardness: f32,
	#[serde(default = "default_false")]
	pub transparent: bool,
	#[serde(default = "default_false")]
	pub empty: bool,
	#[serde(default = "default_true")]
	pub solid: bool,
	#[serde(default = "default_texture")]
	pub texture: String,

	// Generated values
	#[serde(skip)]
	pub uv_rect: Rect,
}

fn default_texture() -> String { 
	"textures/TestTexture.png".to_string()
}

fn default_true() -> bool {
	true
}

fn default_false() -> bool {
	false
}

pub type BlockId = u16;

#[derive(Resource, Default)]
pub struct BlockRegistry {
	definitions: Vec<BlockDefinition>,
	key_to_id: HashMap<String, BlockId>,
}

#[allow(dead_code)]
impl BlockRegistry {
	pub fn get_id(&self, key: &str) -> Option<BlockId> {
		self.key_to_id.get(key).copied()
	}

	pub fn get_def(&self, id: BlockId) -> Option<&BlockDefinition> {
		if id >= self.definitions.len() as u16 {
			return None;
		}

		Some(&self.definitions[id as usize])
	}

	fn register(&mut self, block_def: BlockDefinition) {
		let id: BlockId = self.definitions.len() as BlockId;
		self.key_to_id.insert(block_def.key.clone(), id);
		self.definitions.push(block_def);
	}
}

#[derive(Default, TypePath)]
struct BlockDefinitionAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
enum BlockDefinitionLoadError {
	#[error("Could not load asset: {0}")]
	Io(#[from] std::io::Error),

	#[error("Could not parse RON: {0}")]
	RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for BlockDefinitionAssetLoader {
	type Asset = BlockDefinition;
	type Settings = ();
	type Error = BlockDefinitionLoadError;
	async fn load(
		&self,
		reader: &mut dyn Reader,
		_settings: &(), // tf is this shit man I'm boutta quit
		_load_context: &mut LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes).await?;

		let block_def = ron::de::from_bytes::<BlockDefinition>(&bytes)?;

		Ok(block_def)
	}

	fn extensions(&self) -> &[&str] {
		&["block.ron"]
	}
}

#[derive(Resource, Default)]
struct BlockAssets {
	defs_folder: Handle<LoadedFolder>,
	textures_folder: Handle<LoadedFolder>,
}



fn load_blocks(mut state: ResMut<BlockAssets>, asset_server: Res<AssetServer>) {
	state.defs_folder = asset_server.load_folder("blocks");
	state.textures_folder = asset_server.load_folder("textures");
}

fn build_block_atlas(
	folder: &LoadedFolder,
	images: &mut ResMut<Assets<Image>>,
) -> (TextureAtlasLayout, TextureAtlasSources, Handle<Image>) {
	let mut builder = TextureAtlasBuilder::default();
	builder.padding(uvec2(2, 2));
	
	for handle in folder.handles.iter() {
		let id = handle.id().typed_unchecked::<Image>();
		let Some(texture) = images.get(id) else {
			warn!("Theres a weird ass NOT TEXTURE in the TEXTURES folder ya genius: {:?}", handle.path());
			continue;
		};
		builder.add_texture(Some(id), texture);
	}

	let (layout, sources, atlas_texture) = builder.build().unwrap();
	let texture = images.add(atlas_texture);

	(layout, sources, texture)
}

#[derive(SystemParam)]
struct BlockRegisterParams<'w> {
	block_assets: Res<'w, BlockAssets>,
	loaded_folders: Res<'w, Assets<LoadedFolder>>,
	block_definitions: Res<'w, Assets<BlockDefinition>>,
	asset_server: Res<'w, AssetServer>,
	images: ResMut<'w, Assets<Image>>,
	materials: ResMut<'w, Assets<VoxelMaterial>>,
}

fn register_blocks(
	mut params: BlockRegisterParams,
	mut commands: Commands,
	mut next_state: ResMut<NextState<GameState>>,
	mut registry: ResMut<BlockRegistry>
) {
	let Some(texture_folder) = params.loaded_folders.get(&params.block_assets.textures_folder) else {
		return;
	};

	let Some(blocks_folder) = params.loaded_folders.get(&params.block_assets.defs_folder) else {
		return;
	};
	
	next_state.set(GameState::Playing);
	
	let (layout, sources, atlas) = build_block_atlas(texture_folder, &mut params.images);

	for handle in &blocks_folder.handles {
		let typed_handle = handle.id().typed::<BlockDefinition>();
		let Some(def) = params.block_definitions.get(typed_handle) else {
			continue;
		};

		let Some(texture): Option<Handle<Image>> = params.asset_server.get_handle(def.texture.clone()) else {
			warn!("Texture not available: {}! Are you sure its in the \\textures folder? The blockdef {} has been skipped", def.texture, def.key);
			continue;
		};

		let Some(uv_rect) = sources.uv_rect(&layout, &texture) else {
			warn!("Texture not part of atlas: {}! The blockdef {} has been skipped.", def.texture, def.key);
			continue;
		};

		let mut registered_def = def.clone();
		registered_def.uv_rect = uv_rect;

		registry.register(registered_def);
	}

	info!("Blockdefs have been registered!");

	let material_handle = params.materials.add(
		ExtendedMaterial {
			base: StandardMaterial::default(),
			extension: VoxelMaterialExtension { atlas },
		}
	);

	commands.insert_resource(ChunkMaterialRes(material_handle));

	info!("Material resource created!");
}