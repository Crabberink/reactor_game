use crate::player::protocol::PlayerProtocolPlugin;
use bevy::prelude::*;
use lightyear::prelude::*;
use crate::voxels::protocol::VoxelsProtocolPlugin;

pub struct ProtocolPlugin;
impl Plugin for ProtocolPlugin {
	fn build(&self, app: &mut App) {
		app.component::<Name>().replicate();

		app.add_plugins(PlayerProtocolPlugin);
		app.add_plugins(VoxelsProtocolPlugin);
	}
}