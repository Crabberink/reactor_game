use bevy::{prelude::*, window::{CursorGrabMode, CursorOptions}};
use leafwing_input_manager::prelude::*;
use crate::{GameState, chunks::ChunkLoader};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
        app
			.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(OnEnter(GameState::Playing), setup_player)
			.add_systems(Update, player_movement.run_if(in_state(GameState::Playing)));
	}
}

#[derive(Component)]
pub struct MoveSpeed {
	speed: f32,
}

#[derive(Component)]
pub struct PlayerMovement;

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq, Debug)]
enum PlayerAction {
	#[actionlike(DualAxis)]
	Move,
	#[actionlike(DualAxis)]
	Look,
	Jump
}

fn setup_player(mut commands: Commands) {
	commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-12.0, 12.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ChunkLoader::of_range(1),
		PlayerMovement,
		MoveSpeed { speed: 5.0 },
		InputMap::<PlayerAction>::default()
			.with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
			.with_dual_axis(PlayerAction::Move, GamepadStick::LEFT)
			.with_dual_axis(PlayerAction::Look, MouseMove::default())
			.with_dual_axis(PlayerAction::Look, GamepadStick::RIGHT)
    ));
}

const LOOK_SENSITIVITY: f32 = 0.005;

fn player_movement(
	time: Res<Time>,
	mut window: Single<&mut Window>,
	mut cursor_options: Single<&mut CursorOptions>,
	mut players: Query<(&mut Transform, &MoveSpeed, &ActionState<PlayerAction>), With<PlayerMovement>>
) {
	let cursor_pos = window.size() / 2.0;

	window.set_cursor_position(Some(cursor_pos));
	cursor_options.grab_mode = CursorGrabMode::Locked;

	for (mut transform, move_speed, actions) in players.iter_mut() {
		let move_axis = actions.clamped_axis_pair(&PlayerAction::Move);
		let forward = transform.forward();
		let right = transform.right();

		transform.translation += (forward * move_axis.y + right * move_axis.x) * move_speed.speed * time.delta_secs();
		
		let look_axis = actions.axis_pair(&PlayerAction::Look);
		transform.rotate_y(-look_axis.x * LOOK_SENSITIVITY);
		transform.rotate_local_x(-look_axis.y * LOOK_SENSITIVITY);

		if actions.just_pressed(&PlayerAction::Jump) {
			transform.translation.y += 3.0;
		}
	}
}