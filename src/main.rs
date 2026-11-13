mod voxels;
mod blockdefs;
mod chunks;
mod player;
mod network;

use bevy::render::error_handler::{ErrorType, RenderErrorHandler, RenderErrorPolicy};
use bevy::prelude::*;
use avian3d::prelude::*;
use bevy::app::{PanicHandlerPlugin, TerminalCtrlCHandlerPlugin};
use bevy::diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::input::InputPlugin;
use bevy::log::LogPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use clap::{Parser, ValueEnum};
use lightyear::avian3d::plugin::{AvianReplicationMode, LightyearAvianPlugin};
use voxels::VoxelsPlugin;
use blockdefs::BlocksPlugin;

use crate::chunks::{ChunksPlugin};
use crate::network::client::{ClientNetworkPlugin, LocalClientId};
use crate::network::server::ServerNetworkPlugin;
use crate::network::shared::SharedNetworkPlugin;
use crate::player::client::ClientPlayerPlugin;
use crate::player::server::ServerPlayerPlugin;
use crate::player::shared::SharedPlayerPlugin;

#[derive(Parser)]
struct Cli {
    #[arg(value_enum)]
    mode: Mode,

    #[arg(long, default_value_t = 0)]
    client_id: u64,
}

#[derive(Clone, ValueEnum)]
enum Mode {
    ClientServer,
    Client,
    Server,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();

    app.insert_resource(LocalClientId(cli.client_id));

    match cli.mode {
        Mode::Server => {
            app.add_plugins(DefaultPlugins);

            app.add_plugins(ServerPlugin);
            app.add_plugins(SharedPlugin);
        }
        Mode::Client => {
            app.add_plugins(DefaultPlugins);

            app.add_plugins(ClientPlugin);
            app.add_plugins(SharedPlugin);
        }
        Mode::ClientServer => {
            app.add_plugins(DefaultPlugins);

            app.add_plugins(ServerPlugin);
            app.add_plugins(ClientPlugin);
            app.add_plugins(SharedPlugin);
        }
    }

    app.run();
}

pub struct SharedPlugin;
pub struct ClientPlugin;
pub struct ServerPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(EguiPlugin::default())
            .add_plugins(WorldInspectorPlugin::new());

        app.add_plugins(SharedNetworkPlugin);

        app.add_plugins(LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Position { sync_to_transform: false },
            rollback_resources: false,
            ..default()
        });

        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                // Lightyear avian handles transform sync
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                // Sleeping messes with rollbacks
                .disable::<IslandPlugin>()
                .disable::<IslandSleepingPlugin>(),
        );

        app.insert_resource(RenderErrorHandler(
            |error, _main_world, _render_world| match error.ty {
                ErrorType::Validation => RenderErrorPolicy::Ignore,
                _ => RenderErrorPolicy::Ignore,
            }
        ));

        app.init_state::<GameState>();


        app
            // .add_plugins(BlocksPlugin)
            // .add_plugins(VoxelsPlugin)
            // .add_plugins(ChunksPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, setup);

        app.add_plugins(SharedPlayerPlugin);
    }
}

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ClientNetworkPlugin);
        app.add_plugins(ClientPlayerPlugin);
    }
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ServerNetworkPlugin);
        app.add_plugins(ServerPlayerPlugin);
    }
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    #[default]
    Loading,
    Playing
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    
    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 8.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    
    im_so_fucking_sorry(&mut commands, &mut meshes, &mut materials);
}

fn im_so_fucking_sorry(
    mut commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>
) {
    commands.spawn(Camera3d::default());

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(-9.0, 0.0, 0.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(9.0, 0.0, 0.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, 9.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, -9.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(-9.0, 3.0, 0.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(9.0, 3.0, 0.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 3.0, 9.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 3.0, -9.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(-9.0, -3.0, 0.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(9.0, -3.0, 0.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, -3.0, 9.0)
    ));

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, -3.0, -9.0)
    ));

}