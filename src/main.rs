pub mod voxels;
pub mod blockdefs;

use bevy::{
    prelude::*,
};

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_flycam::PlayerPlugin;
use voxels::VoxelsPlugin;
use blockdefs::BlocksPlugin;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_plugins(BlocksPlugin)
        .add_plugins(VoxelsPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, fps_system.run_if(in_state(GameState::Playing)))
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

    let mut debug_voxel_data = [[[0; 16]; 16]; 16];
    for (x, plane) in debug_voxel_data.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
        for (y, row) in plane.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
            for (z, voxel) in row.iter_mut().enumerate().take(voxels::CHUNK_SIZE) {
                *voxel = if (x & y & z) == 0 { 1 } else { 0 };
            }
        }
    }

    for x in 0usize..10 {
        for y in 0usize..10 {
            for z in 0usize..10 {
                commands.spawn((
                    voxels::Chunk {
                        coord: IVec3::new(x as i32, y as i32, z as i32),
                        voxels: debug_voxel_data,
                    },  
                    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                    Transform::from_xyz(
                        (x * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE,
                        (y * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE,
                        (z * voxels::CHUNK_SIZE) as f32 * voxels::VOXEL_SIZE
                    ),
                    MeshMaterial3d(materials.add(Color::hsl(0.0, 1.0, 1.0))),
                    voxels::DirtyChunk,
                ));
            }
        }
    }

    


    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 8.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-20.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    // ));
}

