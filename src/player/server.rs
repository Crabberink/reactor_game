use crate::block_id_palette;
use crate::blockdefs::BlockRegistry;
use crate::chunks::{ChunkLoader, ChunkStream};
use crate::player::PlayerBody;
use crate::player::shared::{MoveSpeed, PLAYER_HEIGHT, PlayerAction, PlayerPhysicsBundle, apply_character_action};
use crate::voxels::protocol::Chunk;
use crate::voxels::{VoxelWorld, world_to_block};
use avian3d::prelude::*;
use bevy::app::{App, Plugin};
use bevy::log::info;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use lightyear::connection::client_of::ClientOf;
use lightyear::connection::host::HostServer;
use lightyear::prelude::*;

pub struct ServerPlayerPlugin;
impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_connected_player);
        app.add_systems(FixedUpdate, handle_player_actions);
    }
}

/// Spawns a player for a connected client
fn handle_connected_player(
    trigger: On<Add<Connected>>,
    client_query: Query<&RemoteId, With<ClientOf>>,
    player_query: Query<Entity, With<PlayerBody>>,
    mut commands: Commands,
) {
    let Ok(remote_id) = client_query.get(trigger.entity) else {
        return;
    };

    let remote_id = remote_id.0;

    info!("Spawning player for client {remote_id:?}");

    // Lets us decide color/position based on existing number of players
    let num_players = player_query.iter().count();

    let spawn_pos = Vec3::new((num_players * 3) as f32, 0.0, 0.0);

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
        ChunkStream::default(),
        ControlledBy {
            owner: trigger.entity,
            // They disappear when the player leaves
            lifetime: Lifetime::SessionBased,
        },
    ));


}

fn handle_player_actions(
    time: Res<Time>,
    block_registry: Res<BlockRegistry>,
    spatial_query: SpatialQuery,
    host_server: Query<(), With<HostServer>>,
    chunks: Query<Entity, (With<Chunk>, Without<MoveSpeed>)>,
    mut player_query: Query<(
        Entity,
        Forces,
        &ComputedMass,
        &ActionState<PlayerAction>,
        &MoveSpeed,
        Has<Predicted>
    )>,
    mut voxel_world: VoxelWorld,
) {
    block_id_palette!(block_registry, {
		"rg_air" => air_id,
        "rg_zart" => zart_id,
	});

    let is_host_server = !host_server.is_empty();

    for (entity, forces, computed_mass, action_state, move_speed, predicted) in player_query.iter_mut() {
        if is_host_server && predicted { continue; }

        // gotta grab this shit for later
        let origin = forces.position() + Vec3::Y * (PLAYER_HEIGHT / 2.0);

        apply_character_action(&time, &spatial_query, entity, forces, computed_mass, action_state, move_speed);

        let look_axes = action_state.axis_pair(&PlayerAction::LookAxes);
        let look_quat = Quat::from_euler(EulerRot::YXZ, look_axes.x, look_axes.y, 0.0);
        let look_dir = look_quat * Dir3::NEG_Z;


        let Some(ray_hit_data) = spatial_query.cast_ray(
            origin,
            look_dir,
            5.0
            , true,
            &SpatialQueryFilter::default().with_excluded_entities([entity])
        ) else { continue; };

        if !chunks.contains(ray_hit_data.entity) { continue; }

        let hit_point = origin + look_dir * ray_hit_data.distance;

        if action_state.just_pressed(&PlayerAction::Break) {
            let block_position = world_to_block(hit_point - (ray_hit_data.normal * 0.5));

        voxel_world.set_block(block_position, air_id);
        }

        if action_state.just_pressed(&PlayerAction::Place) {
            let block_position = world_to_block(hit_point + (ray_hit_data.normal * 0.5));

            voxel_world.set_block(block_position, zart_id);
        }
    }
}