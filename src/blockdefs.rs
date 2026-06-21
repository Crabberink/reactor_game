use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy::asset::{Asset, AssetLoader, LoadContext, LoadedFolder};
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

use crate::GameState;

pub struct BlocksPlugin;
impl Plugin for BlocksPlugin {
	fn build(&self, app: &mut App) {
		app
			.init_resource::<BlockAssets>()
			.init_resource::<BlockRegistry>()
			.init_asset::<BlockDefinition>()
			.init_asset_loader::<BlockDefinitionAssetLoader>()
			.add_systems(Startup, load_blocks)
			.add_systems(Update, register_blocks);
	}
}

#[allow(dead_code)]
#[derive(Asset, TypePath, Deserialize, Clone)]
pub struct BlockDefinition {
	pub name: String,
	pub key: String,
	#[serde(default)]
	pub hardness: f32,
	#[serde(default = "default_transparent")]
	pub transparent: bool,
	#[serde(default = "default_transparent")]
	pub empty: bool,
}

fn default_transparent() -> bool {
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

	pub fn get_def(&self, id: BlockId) -> &BlockDefinition {
		&self.definitions[id as usize]
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
	folder: Handle<LoadedFolder>,
}

fn load_blocks(mut state: ResMut<BlockAssets>, asset_server: Res<AssetServer>) {
	state.folder = asset_server.load_folder("blocks");
}

fn register_blocks(
	block_assets: Res<BlockAssets>,
	loaded_folders: Res<Assets<LoadedFolder>>,
	block_definitions: Res<Assets<BlockDefinition>>,
	mut next_state: ResMut<NextState<GameState>>,
	mut registry: ResMut<BlockRegistry>
) {
	let Some(folder) = loaded_folders.get(&block_assets.folder) else {
		return;
	};

	next_state.set(GameState::Playing);

	for handle in &folder.handles {
		let typed_handle = handle.id().typed::<BlockDefinition>();
		let Some(def) = block_definitions.get(typed_handle) else {
			continue;
		};

		registry.register(def.clone());
	}
}