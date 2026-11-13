use crate::player::shared::{MoveSpeed, PlayerAction};
use crate::player::{PlayerBody, PlayerHead};
use bevy::app::{App, Plugin};
use bevy::utils::default;
use lightyear::input::prelude::InputConfig;
use lightyear::prelude::AppComponentExt;
use lightyear::prelude::input::leafwing;

pub struct PlayerProtocolPlugin;

impl Plugin for PlayerProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(leafwing::InputPlugin::<PlayerAction> {
            config: InputConfig::<PlayerAction> {
                rebroadcast_inputs: true,
                ..default()
            },
        });

        app.component::<MoveSpeed>().replicate();
        app.component::<PlayerHead>().replicate();
        app.component::<PlayerBody>().replicate();
    }
}
