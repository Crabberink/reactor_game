use std::cmp::Ordering;
use std::time::Duration;
use avian3d::prelude::*;
use avian3d::prelude::forces::ForcesItem;
use bevy::app::{App, FixedUpdate, Plugin, Update};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use crate::GameState;
use crate::player::{PlayerAction, check_input, PlayerMovement, PlayerHead, LOOK_SENSITIVITY, PlayerBody};
use crate::player::protocol::PlayerProtocolPlugin;

pub struct SharedPlayerPlugin;
impl Plugin for SharedPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerProtocolPlugin);
        app.add_systems(Update, debug_sync);
    }
}

pub const PLAYER_HEIGHT: f32 = 1.8;
pub const PLAYER_RADIUS: f32 = 0.4;
pub const GROUNDED_CHECK_RADIUS : f32 = 0.4;

#[derive(Bundle)]
pub struct PlayerPhysicsBundle {
    rigid_body: RigidBody,
    collider: Collider,
    locked_axes: LockedAxes,
    friction: Friction,
    gravity_scale: GravityScale,
}

impl Default for PlayerPhysicsBundle {
    fn default() -> Self {
        PlayerPhysicsBundle {
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cylinder(PLAYER_RADIUS, PLAYER_HEIGHT),
            locked_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            friction: Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            gravity_scale: GravityScale(0.0),
        }
    }
}


// Yo I PROMISE the magic numbers are better this way constants are for LOSERS (ignoring the existing constants)
// ShapeCaster::new(
// 	Collider::cylinder(GROUNDED_CHECK_RADIUS, 0.1),
// 	Vec3::new(0.0, (-PLAYER_HEIGHT / 2.0) + 0.05, 0.0),
// 	Quat::IDENTITY,
// 	Dir3::Y,
// ).with_max_distance(0.1).with_ignore_self(true),

#[derive(Component, Serialize, Deserialize)]
pub struct MoveSpeed {
    pub speed: f32,
    pub max_acceleration: f32,
}

/// Helper function that applies actions to the player
pub fn apply_character_action(
    time: &Res<Time>,
    spatial_query: &SpatialQuery,
    entity: Entity,
    mut forces: ForcesItem,
    computed_mass: &ComputedMass,
    action_state: &ActionState<PlayerAction>,
    move_speed: &MoveSpeed,
) {
    if action_state.just_pressed(&PlayerAction::Jump) {
        let ray_cast_origin = forces.position() + Vec3::new(
            0.0,
            0.01 - PLAYER_HEIGHT / 2.0,
            0.0,
        );

        if spatial_query
            .cast_shape(
                &Collider::sphere(GROUNDED_CHECK_RADIUS),
                ray_cast_origin,
                Quat::IDENTITY, // its a sphere rotation doesn't exist sorta
                Dir3::NEG_Y,
                &ShapeCastConfig::from_max_distance(0.1),
                &SpatialQueryFilter::from_excluded_entities([entity]),
            ).is_some()
        {
            forces.apply_linear_impulse(Vec3::new(0.0,5.0,0.0));
        }
    }

    // Normalized input
    let move_axis = action_state.clamped_axis_pair(&PlayerAction::Move).clamp_length_max(1.0);

    let player_rotation = forces.rotation();
    let forward = player_rotation * Vec3::NEG_Z; // Neg z is forward
    let forward = forward.with_y(0.0).normalize_or_zero(); // y is set to 0.0 to project it to the ground
    let right = player_rotation * Vec3::X; // Pos x is right
    let right = right.with_y(0.0).normalize_or_zero(); // y is set to 0.0 to project it to the ground

    let target_vel = (forward * move_axis.y + right * move_axis.x).clamp_length_max(1.0) * move_speed.speed;

    let ground_vel = forces.linear_velocity().with_y(0.0);

    // If we are moving at max acceleration, how much could our velocity change since the last update?
    let max_velocity_delta = move_speed.max_acceleration * time.delta_secs();

    // What would my velocity be if I had accelerated towards my target over the time step
    let new_ground_vel = ground_vel.move_towards(target_vel, max_velocity_delta);

    // How much acceleration to get myself from my stored velocity to what my velocity for this tick should be
    let accel = (new_ground_vel - ground_vel) / time.delta_secs();

    // info!(
    //     ?move_axis,
    //     ?player_rotation,
    //     ?target_vel,
    //     ?ground_vel,
    //     ?new_ground_vel,
    //     ?accel,
    //     delta = time.delta_secs(),
    //     mass = computed_mass.value(),
    //     "Movement calculation"
    // );

    // Thank you Mr. Eckel
    forces.apply_force(accel * computed_mass.value());
}

fn debug_sync(q: Query<(Entity, &Position, &LinearVelocity), With<PlayerBody>>) {
    let mut rows: Vec<_> = q.iter().collect();
    rows.sort_by_key(|(entity, _, _)| *entity);

    let lines: Vec<String> = rows
        .iter()
        .map(|(e, pos, vel)| format!("{e:?} pos={:.3?} vel={:.3?}", pos.0, vel.0))
        .collect();

    info!("PlayerBody state:\n{}", lines.join("\n"));
}