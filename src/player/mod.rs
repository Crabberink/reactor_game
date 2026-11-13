pub mod client;
pub mod shared;
pub mod server;
pub mod protocol;

use avian3d::{prelude::*};
use bevy::{prelude::*, window::{CursorGrabMode, CursorOptions}};
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use crate::{GameState, blockdefs::BlockRegistry, chunks::ChunkLoader, block_id_palette, voxels::{self, VoxelWorld}};

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerMovement {
	head: Entity,
	head_azimuth: f32,
}

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerHead;

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerBody;

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Copy)]
enum PlayerAction {
	#[actionlike(DualAxis)]
	Move,
	#[actionlike(DualAxis)]
	Look,
	Jump,
	Break,
	Place,
}

const LOOK_SENSITIVITY: f32 = 0.005;

// Stores inputs for FixedUpdate 
#[derive(Component, Default)]
pub struct Inputs {
	jump: bool,
	place: bool,
	breaking: bool,
}

impl Inputs {
	fn register_jump(&mut self, value: bool) {
		if value {
			self.jump = true;
		}
	}
	fn take_jump(&mut self) -> bool {
		let value = self.jump;

		self.jump = false;
		
		value
	}
	fn register_place(&mut self, value: bool) {
		if value { self.place = true; }
	}
	fn take_place(&mut self) -> bool {
		let value = self.place;

		self.place = false;

		value
	}
	fn register_breaking(&mut self, value: bool) {
		if value { self.breaking = true; }
	}
	fn take_breaking(&mut self) -> bool {
		let value = self.breaking;

		self.breaking = false;

		value
	}
}

fn check_input(
	mut players: Query<(
		&mut Inputs,
		&ActionState<PlayerAction>
	), With<PlayerMovement>>
) {
	for (mut inputs, actions) in players.iter_mut() {
		inputs.register_jump(actions.just_pressed(&PlayerAction::Jump));
		inputs.register_place(actions.just_pressed(&PlayerAction::Place));
		inputs.register_breaking(actions.just_pressed(&PlayerAction::Break));
	}
}

fn lock_cursor(
	mut window: Single<&mut Window>,
	mut cursor_options: Single<&mut CursorOptions>,
) {
	let cursor_pos = window.size() / 2.0;

	window.set_cursor_position(Some(cursor_pos));

	cursor_options.grab_mode = CursorGrabMode::Locked;
}

fn block_breaking(
	block_registry: Res<BlockRegistry>,
	mut players: Query<(
		&mut Inputs,
		&PlayerMovement,
	), Without<PlayerHead>>,
	player_heads: Query<(&RayCaster, &RayHits), With<PlayerHead>>,
	mut voxel_world: VoxelWorld,
) {
	block_id_palette!(block_registry, {
		"rg_air" => air_id,
	});

	for (mut inputs, player_movement) in players.iter_mut() {

		if !inputs.take_breaking() { continue; }

		let Ok((ray_caster, ray_hits)) = player_heads.get(player_movement.head) else { continue; };
		let Some(hit) = ray_hits.iter().next() else { continue; };

		let hit_position = (hit.distance + 0.01) * ray_caster.global_direction() + ray_caster.global_origin();

		let block_positon = voxels::world_to_block(hit_position);

		voxel_world.set_block(block_positon, air_id);
	}
}

fn block_placing(
	block_registry: Res<BlockRegistry>,
	mut players: Query<(
		&mut Inputs,
		&PlayerMovement,
	), Without<PlayerHead>>,
	player_heads: Query<(&RayCaster, &RayHits), With<PlayerHead>>,
	mut voxel_world: VoxelWorld,
) {
	block_id_palette!(block_registry, {
		"rg_dirt" => dirt_id,
	});

	for (mut inputs, player_movement) in players.iter_mut() {

		if !inputs.take_place() { continue; }

		let Ok((ray_caster, ray_hits)) = player_heads.get(player_movement.head) else { continue; };
		let Some(hit) = ray_hits.iter().next() else { continue; };

		let hit_position = (hit.distance - 0.01) * ray_caster.global_direction() + ray_caster.global_origin();

		let block_positon = voxels::world_to_block(hit_position);

		voxel_world.set_block(block_positon, dirt_id);
	}
}