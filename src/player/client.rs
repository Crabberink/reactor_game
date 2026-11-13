use avian3d::prelude::{ComputedMass, Forces, Position, SpatialQuery};
use bevy::app::{App, Plugin};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::prelude::{GamepadStick, InputMap, MouseMove, VirtualDPad};
use lightyear::prelude::*;
use crate::player::{PlayerBody, PlayerAction};
use crate::player::shared::{apply_character_action, MoveSpeed, PlayerPhysicsBundle, PLAYER_HEIGHT, PLAYER_RADIUS};

pub struct ClientPlayerPlugin;
impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, handle_player_actions);
        app.add_systems(Update, handle_new_player);
        app.add_observer(handle_controlled_player);
    }
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
            info!("Adding camera to client player {entity:?}");

            commands.entity(entity).insert(Camera3d::default());
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
    mut player_query: Query<
        (
            Entity,
            Forces,
            &ComputedMass,
            &ActionState<PlayerAction>,
            &MoveSpeed,
            Has<Predicted>
        ),
        Or<(With<Predicted>, With<DeterministicPredicted>)>
    >,
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