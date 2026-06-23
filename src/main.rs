pub mod voxels;
pub mod blockdefs;
pub mod chunks;

use bevy::render::error_handler::{ErrorType, RenderErrorHandler, RenderErrorPolicy};
use bevy::{
    prelude::*,
};

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use voxels::VoxelsPlugin;
use blockdefs::BlocksPlugin;

use crate::chunks::{ChunkLoader, ChunksPlugin};


fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
        )
        .insert_resource(RenderErrorHandler(
            |error, _main_world, _render_world| match error.ty {
                ErrorType::Validation => RenderErrorPolicy::Ignore,
                _ => RenderErrorPolicy::Ignore,
            }
        ))
        .init_state::<GameState>()
        .add_plugins(BlocksPlugin)
        .add_plugins(VoxelsPlugin)
        .add_plugins(ChunksPlugin)
        // .add_plugins(PlayerPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, fps_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, forward_system.run_if(in_state(GameState::Playing)))
        .run();
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum GameState {
    #[default]
    Loading,
    Playing
}

fn fps_system(diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) && let Some(value) = fps.smoothed() {
        println!("{:.1} fps", value);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 1.8))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.0, 0.0)
    ));

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

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-12.0, 12.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        ChunkLoader::of_range(1),
    ));
}

fn forward_system(mut players: Query<&mut Transform, With<Camera3d>>) {
    for mut transform in players.iter_mut() {
        let forward: Vec3 = transform.forward().into();
        transform.translation += forward * 4.5;
    }
}