use avian3d::{prelude::*};
use bevy::{prelude::*, window::{CursorGrabMode, CursorOptions}};
use leafwing_input_manager::prelude::*;
use crate::{GameState, chunks::ChunkLoader};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
        app
			.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(GameState::Playing), setup_player)
			.add_systems(Update, check_input)
			.add_systems(FixedUpdate, player_movement.run_if(in_state(GameState::Playing)));
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
	Jump
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
        Transform::from_xyz(-12.0, 12.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ChunkLoader::of_range(1),
		PlayerMovement {
			head,
			head_azimuth: 0.0,
		},
		MoveSpeed { speed: 5.0, accel_sharpness: 10.0 },
		Inputs { jump: false },
		RigidBody::Dynamic,
		Collider::cylinder(1.0, PLAYER_HEIGHT),
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
			.with(PlayerAction::Jump, GamepadButton::South),
    )).id();

	commands.entity(head).insert(ChildOf(player));
}

const LOOK_SENSITIVITY: f32 = 0.005;

// Stores inputs for FixedUpdate 
#[derive(Component)]
pub struct Inputs {
	jump: bool,
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
}

fn check_input(
	mut players: Query<(
		&mut Inputs,
		&ActionState<PlayerAction>
	), With<PlayerMovement>>
) {
	
	for (mut inputs, actions) in players.iter_mut() {
		inputs.register_jump(actions.just_pressed(&PlayerAction::Jump));
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
			info!("Jumping");
			forces.apply_linear_acceleration(Vec3::new(0.0, 200.0, 0.0));
		}
	}

}