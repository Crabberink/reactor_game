use bevy::prelude::*;
use lightyear::prelude::*;
use crate::player::protocol::PlayerProtocolPlugin;

pub struct ProtocolPlugin;
impl Plugin for ProtocolPlugin {
	fn build(&self, app: &mut App) {
		app.component::<Name>().replicate();

		app.add_plugins(PlayerProtocolPlugin);
	}
}