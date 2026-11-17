pub mod client;
pub mod shared;
pub mod server;
pub mod protocol;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerHead;

#[derive(Component, Serialize, Deserialize)]
pub struct PlayerBody;

const LOOK_SENSITIVITY: f32 = 0.005;