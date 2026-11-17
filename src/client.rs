use crate::player::client::ClientPlayerPlugin;
use crate::voxels::client::ClientVoxelsPlugin;
use avian3d::parry::glamx::Vec3;
use bevy::app::{App, Plugin, Startup};
use bevy::light::{DirectionalLight, PointLight};
use bevy::prelude::{Commands, Transform, default};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);

        app.add_plugins(ClientPlayerPlugin);
        app.add_plugins(ClientVoxelsPlugin);
    }
}

fn setup(
    mut commands: Commands,
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
}