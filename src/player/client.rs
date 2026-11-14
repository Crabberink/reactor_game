use std::f32::consts::PI;
use crate::player::shared::{MoveSpeed, PLAYER_HEIGHT, PLAYER_RADIUS, PlayerAction, PlayerPhysicsBundle, apply_character_action};
use crate::player::{PlayerBody, PlayerHead, LOOK_SENSITIVITY};
use avian3d::prelude::{ComputedMass, Forces, Position, SpatialQuery};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::prelude::{GamepadStick, InputMap, MouseMove, VirtualDPad};
use lightyear::prelude::*;
use crate::LoadingState;

pub struct ClientPlayerPlugin;
impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_controlled_player);

        app.add_systems(Update, handle_new_player);
        app.add_systems(FixedUpdate, handle_player_actions);
        app.add_systems(Update, (apply_head_pitch, update_view_angles));
        app.add_systems(FixedPreUpdate, write_look_axes);
        app.add_systems(OnEnter(LoadingState::Loaded), lock_mouse);
    }
}

#[derive(Component)]
struct ViewAngles {
    pitch: f32,
    yaw: f32,
}

/// Adds physics and mesh to newly created predicted players
fn handle_new_player(
    mut player_query: Query<(Entity, Has<Controlled>), (Added<Predicted>, With<PlayerBody>)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, is_client) in player_query.iter_mut() {
        info!("Adding physics to new player {entity:?}");

        commands.entity(entity).insert(PlayerPhysicsBundle::default());

        if is_client {
            info!("Adding head to client player {entity:?}");

            commands.entity(entity).insert(ViewAngles { pitch: 0.0, yaw: 0.0 });
            commands.spawn((
                PlayerHead,
                Camera3d::default(),
                Transform::from_xyz(0.0, PLAYER_HEIGHT / 2.0, 0.0),
                ChildOf(entity),
            ));

            continue;
        }

        info!("Adding mesh to new player {entity:?}");
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(
                PLAYER_RADIUS, PLAYER_HEIGHT - PLAYER_RADIUS,
            ))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0)))
        ));
    }
}

/// Adds an input map to controlled players
fn handle_controlled_player(
    trigger: On<Add<Controlled>>,
    player_query: Query<(), (With<PlayerBody>, Without<InputMap<PlayerAction>>)>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    if player_query.get(entity).is_err() { return; }

    info!("Adding input map to controlled player {entity:?}");
    commands.entity(entity).insert(player_input_map());
}

fn player_input_map() -> InputMap<PlayerAction> {
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
}

fn handle_player_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    // Excludes host-server client
    timeline: Res<LocalTimeline>,
    mut player_query: Query<(
        Entity,
        Forces,
        &ComputedMass,
        &ActionState<PlayerAction>,
        &MoveSpeed,
        Has<Predicted>
    ), Or<(With<Predicted>, With<DeterministicPredicted>)>>,
    mut player_head_query: Query<&mut Transform, With<PlayerHead>>,
) {
    let tick = timeline.tick();

    for (
        entity,
        forces,
        computed_mass,
        action_state,
        move_speed, predicted
    ) in player_query.iter_mut() {
        apply_character_action(&time, &spatial_query, entity, forces, computed_mass, action_state, move_speed);
    }
}

fn apply_head_pitch(
    players: Query<&ViewAngles>,
    mut heads: Query<(&ChildOf, &mut Transform), With<PlayerHead>>,
) {
    for (child_of, mut tf) in &mut heads {
        if let Ok(view_angles) = players.get(child_of.parent()) {
            tf.rotation = Quat::from_rotation_x(view_angles.pitch);
        }
    }
}

fn update_view_angles(
    mut query: Query<(&ActionState<PlayerAction>, &mut ViewAngles)>,
) {
    for (action_state, mut view_angles) in query.iter_mut() {
        let delta = action_state.axis_pair(&PlayerAction::Look);

        view_angles.yaw -= delta.x * LOOK_SENSITIVITY;
        view_angles.pitch = (view_angles.pitch - delta.y * LOOK_SENSITIVITY).clamp(0.001 - PI / 2.0,PI / 2.0 - 0.001);
    }
}

fn write_look_axes(
    mut query: Query<(&mut ActionState<PlayerAction>, &ViewAngles)>,
) {
    for (mut action_state, view_angles) in query.iter_mut() {
        action_state.set_axis_pair(&PlayerAction::LookAxes, Vec2::new(view_angles.yaw, view_angles.pitch));
    }
}

fn lock_mouse(
    mut cursor_options: Single<&mut CursorOptions>,
) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
}