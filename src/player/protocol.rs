use bevy::app::{App, Plugin};
use bevy::utils::default;
use lightyear::input::prelude::InputConfig;
use lightyear::prelude::AppComponentExt;
use lightyear::prelude::input::leafwing;
use crate::player::{PlayerAction, PlayerBody, PlayerHead};
use crate::player::shared::MoveSpeed;

pub struct PlayerProtocolPlugin;

impl Plugin for PlayerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.component::<MoveSpeed>().replicate();
        app.component::<PlayerHead>().replicate();
        app.component::<PlayerBody>().replicate();

        app.add_plugins(leafwing::InputPlugin::<PlayerAction> {
            config: InputConfig::<PlayerAction> {
                rebroadcast_inputs: true,
                ..default()
            },
        });
    }
}
