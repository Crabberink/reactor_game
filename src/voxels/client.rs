use crate::voxels::mesh::VoxelMeshingPlugin;
use bevy::prelude::*;

pub struct ClientVoxelsPlugin;

impl Plugin for ClientVoxelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VoxelMeshingPlugin);
    }
}