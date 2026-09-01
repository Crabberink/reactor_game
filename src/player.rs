use avian3d::{prelude::*};
use bevy::{prelude::*, window::{CursorGrabMode, CursorOptions}};
use leafwing_input_manager::prelude::*;
use crate::{GameState, blockdefs::BlockRegistry, chunks::ChunkLoader, voxels::{self, VoxelWorld}};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
        app
			.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(GameState::Playing), setup_player)
			.add_systems(Update, check_input)
			.add_systems(FixedUpdate, (block_placing, block_breaking, player_movement).chain().run_if(in_state(GameState::Playing)));
	}
}

#[derive(Component)]
pub struct MoveSpeed {
	speed: f32,
	accel_sharpness: f32,
}

#[derive(Component)]
pub struct PlayerMovement {
	head: Entity,
	head_azimuth: f32,
}

#[derive(Component)]
pub struct PlayerCamera;


#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq, Debug)]
enum PlayerAction {
	#[actionlike(DualAxis)]
	Move,
	#[actionlike(DualAxis)]
	Look,
	Jump,
	Break,
	Place,
}

const PLAYER_HEIGHT: f32 = 1.8;
const GROUNDED_CHECK_RADIUS : f32 = 0.4;

fn setup_player(mut commands: Commands) {
	let head = commands.spawn((
		Camera3d::default(),
		PlayerCamera,
		Transform::from_xyz(0.0, (PLAYER_HEIGHT / 2.0) - 0.1, 0.0),
	)).id();

	let player = commands.spawn((
        Transform::from_xyz(-12.0, 12.0, 12.0).looking_to(Dir3::NEG_Z,Dir3::Y),
        ChunkLoader::of_range(5),
		PlayerMovement {
			head,
			head_azimuth: 0.0,
		},
		MoveSpeed { speed: 5.0, accel_sharpness: 10.0 },
		Inputs::default(),
		RigidBody::Dynamic,
		Collider::cylinder(0.4, PLAYER_HEIGHT),
		LockedAxes::new().lock_rotation_x().lock_rotation_y().lock_rotation_z(),
		Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
		// Yo I PROMISE the magic numbers are better this way constants are for LOSERS (ignoring the existing constants)
		ShapeCaster::new(
			Collider::cylinder(GROUNDED_CHECK_RADIUS, 0.1),
			Vec3::new(0.0, (-PLAYER_HEIGHT / 2.0) + 0.05, 0.0),
			Quat::IDENTITY,
			Dir3::Y,
		).with_max_distance(0.1).with_ignore_self(true),
		InputMap::<PlayerAction>::default()
			.with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
			.with_dual_axis(PlayerAction::Move, GamepadStick::LEFT)
			.with_dual_axis(PlayerAction::Look, MouseMove::default())
			.with_dual_axis(PlayerAction::Look, GamepadStick::RIGHT)
			.with(PlayerAction::Jump, KeyCode::Space)
			.with(PlayerAction::Jump, GamepadButton::South)
			.with(PlayerAction::Break, MouseButton::Left)
			.with(PlayerAction::Break, GamepadButton::RightTrigger)
			.with(PlayerAction::Place, MouseButton::Right)
			.with(PlayerAction::Place, GamepadButton::LeftTrigger)
    )).id();

	let raycast_filter = SpatialQueryFilter::default()
		.with_excluded_entities([player, head]);

	commands.entity(head).insert(ChildOf(player));
	commands.entity(head).insert(
		RayCaster::new(Vec3::ZERO, Dir3::NEG_Z)
			.with_ignore_self(true)
			.with_max_distance(5.0)
			.with_max_hits(1)
			.with_query_filter(raycast_filter)
	);
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

#[allow(clippy::complexity)]
fn player_movement(
	_time: Res<Time>,
	mut window: Single<&mut Window>,
	mut cursor_options: Single<&mut CursorOptions>,
	mut players: Query<(
		Forces,
		&mut Transform,
		&mut PlayerMovement,
		&mut Inputs,
		&ShapeHits,
		&MoveSpeed,
		&ActionState<PlayerAction>,
	), Without<PlayerCamera>>,
	mut player_heads: Query<&mut Transform, With<PlayerCamera>>,
) {
	let cursor_pos = window.size() / 2.0;

	window.set_cursor_position(Some(cursor_pos));
	cursor_options.grab_mode = CursorGrabMode::Locked;

	for (mut forces, mut transform, mut player_movement, mut inputs, shape_hits, move_speed, actions) in players.iter_mut() {
		let move_axis = actions.clamped_axis_pair(&PlayerAction::Move);

		let forward = transform.forward();
		let right = transform.right();

		// Movement vectors with no vertical component, so the player doesn't fly when looking up or down
		let horizontal_forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
		let horizontal_right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

		let target_vel = (horizontal_forward * move_axis.y + horizontal_right * move_axis.x) * move_speed.speed;
		let linear_velocity = forces.linear_velocity();

		let current_horizontal_vel = Vec3::new(linear_velocity.x, 0.0, linear_velocity.z);

		let vel_diff = target_vel - current_horizontal_vel;

		let acceleration = vel_diff * move_speed.accel_sharpness;

		forces.apply_linear_acceleration(acceleration);


		let look_axis = actions.axis_pair(&PlayerAction::Look);
		transform.rotate_y(-look_axis.x * LOOK_SENSITIVITY);

		player_movement.head_azimuth += -look_axis.y * LOOK_SENSITIVITY;
		player_movement.head_azimuth = player_movement.head_azimuth.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

		if let Ok(mut head_transform) = player_heads.get_mut(player_movement.head) {
			head_transform.rotation = Quat::from_axis_angle(Vec3::X, player_movement.head_azimuth);
		}

		let grounded = !shape_hits.is_empty();

		if inputs.take_jump() && grounded {
			forces.apply_linear_acceleration(Vec3::new(0.0, 300.0, 0.0));
		}
	}

}

fn block_breaking(
	block_registry: Res<BlockRegistry>,
	mut players: Query<(
		&mut Inputs,
		&PlayerMovement,
	), Without<PlayerCamera>>,
	player_heads: Query<(&RayCaster, &RayHits), With<PlayerCamera>>,
	mut voxel_world: VoxelWorld,
) {
	let Some(air_id) = block_registry.get_id("rg_air") else {
		error!("Failed to get a block id!");
		return;
	};

	for (mut inputs, player_movement) in players.iter_mut() {

		if !inputs.take_breaking() { continue; }

		info!("Yo we breakin");

		let Ok((ray_caster, ray_hits)) = player_heads.get(player_movement.head) else { continue; };
		let Some(hit) = ray_hits.iter().next() else { continue; };

		info!("AND we hittin");

		let hit_position = (hit.distance + 0.01) * ray_caster.global_direction() + ray_caster.global_origin();

		let block_positon = voxels::world_to_block(hit_position);

		let broke = voxel_world.set_block(block_positon, air_id);

		info!("We broke? {}", broke);
	}
}

fn block_placing(
	block_registry: Res<BlockRegistry>,
	mut players: Query<(
		&mut Inputs,
		&PlayerMovement,
	), Without<PlayerCamera>>,
	player_heads: Query<(&RayCaster, &RayHits), With<PlayerCamera>>,
	mut voxel_world: VoxelWorld,
) {
	let Some(dirt_id) = block_registry.get_id("rg_dirt") else {
		error!("Failed to get a block id!");
		return;
	};

	for (mut inputs, player_movement) in players.iter_mut() {

		if !inputs.take_place() { continue; }

		info!("Yo we placin");

		let Ok((ray_caster, ray_hits)) = player_heads.get(player_movement.head) else { continue; };
		let Some(hit) = ray_hits.iter().next() else { continue; };

		info!("AND we hittin");

		let hit_position = (hit.distance - 0.01) * ray_caster.global_direction() + ray_caster.global_origin();

		let block_positon = voxels::world_to_block(hit_position);

		let replaced = voxel_world.set_block(block_positon, dirt_id);

		info!("We place? {}", replaced);
	}
}