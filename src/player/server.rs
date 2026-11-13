use std::time::Duration;
use avian3d::prelude::*;
use avian3d::prelude::forces::ForcesItem;
use lightyear::prelude::*;
use bevy::prelude::*;
use bevy::app::{App, Plugin};
use bevy::log::{info};
use bevy::time::common_conditions::on_timer;
use leafwing_input_manager::action_state::ActionState;
use lightyear::connection::client_of::ClientOf;
use lightyear::connection::host::HostServer;
use crate::chunks::ChunkLoader;
use crate::player::{Inputs, PlayerBody, PlayerAction};
use crate::player::shared::{apply_character_action, PlayerPhysicsBundle, MoveSpeed};

pub struct ServerPlayerPlugin;
impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_connected_player);
        app.add_systems(FixedUpdate, handle_player_actions);
        // app.add_systems(FixedUpdate, test_teleport_system.run_if(on_timer(Duration::from_secs(5))));
    }
}

/// Spawns a player for a connected client
fn handle_connected_player(
    trigger: On<Add<Connected>>,
    client_query: Query<&RemoteId, With<ClientOf>>,
    player_query: Query<Entity, With<PlayerBody>>,
    mut commands: Commands,
) {
    info!("Triggered connected player");
    let Ok(remote_id) = client_query.get(trigger.entity) else {
        return;
    };

    let remote_id = remote_id.0;

    info!("Spawning player for client {remote_id:?}");

    // Lets us decide color/position based on existing number of players
    let num_players = player_query.iter().count();

    let spawn_pos = Vec3::new((num_players * 3) as f32, 0.0, 0.0);

    // let head = commands.spawn((
    //     PlayerHead,
    //     Transform::from_xyz(0.0, (PLAYER_HEIGHT / 2.0) - 0.1, 0.0),
    // )).id();
    // commands.entity(head).insert(ChildOf(player));

    commands.spawn((
        Name::new("Player"),
        Position(spawn_pos),
        PlayerBody,
        ChunkLoader::of_range(5),
        MoveSpeed { speed: 5.0, max_acceleration: 20.0 },
        PlayerPhysicsBundle::default(),
        ActionState::<PlayerAction>::default(),

        Replicate::to_clients(NetworkTarget::All),
        // We want everyone to predict what this player is going to do, because its a physics object
        PredictionTarget::to_clients(NetworkTarget::All),
        // The player shouldn't interpolate itself
        // InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(remote_id)),

        ControlledBy {
            owner: trigger.entity,
            // They disappear when the player leaves
            lifetime: Lifetime::SessionBased,
        },
    ));
}

fn handle_player_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    host_server: Query<(), With<HostServer>>,
    mut player_query: Query<(
        Entity,
        Forces,
        &ComputedMass,
        &ActionState<PlayerAction>,
        &MoveSpeed,
        Has<Predicted>
    )>,
) {
    let is_host_server = !host_server.is_empty();

    for (entity, forces, computed_mass, action_state, move_speed, predicted) in player_query.iter_mut() {
        if is_host_server && predicted { info!("Skipping host server movement"); continue; }

        info!("Applying movement for action state {action_state:?}");
        apply_character_action(&time, &spatial_query, entity, forces, computed_mass, action_state, move_speed);
    }
}

fn test_teleport_system(
    mut query: Query<&mut Position, With<PlayerBody>>,
) {
    info!("Running teleport");

    for mut position in query.iter_mut() {
        position.x += 10.0;
    }
}