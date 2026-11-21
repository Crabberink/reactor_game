mod voxels;
mod blockdefs;
mod chunks;
mod player;
mod network;
mod cli;
mod shared;
mod client;
mod server;

use bevy::prelude::*;
use clap::Parser;
use std::time::Duration;

use crate::cli::*;
use crate::client::ClientPlugin;
use crate::network::shared::FIXED_TIMESTEP_HZ;
use crate::server::ServerPlugin;
use crate::shared::SharedPlugin;

fn main() {
    let cli = Cli::parse();
    let mut app = cli.build_app(Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ), false);

    app.add_plugins(SharedPlugin);

    cli.spawn_connections(&mut app);

    match cli.mode {
        Some(Mode::Client { .. }) => {
            app.add_plugins(ClientPlugin);
            add_input_delay(&mut app);
        },
        Some(Mode::Server) => {
            app.add_plugins(ServerPlugin);
        },
        Some(Mode::HostClient { .. }) => {
            app.add_plugins(ClientPlugin);
            app.add_plugins(ServerPlugin);
            add_input_delay(&mut app);
        },
        _ => {}
    }


    app.run();
}

#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum LoadingState {
    #[default]
    Loading,
    Loaded
}

fn add_input_delay(app: &mut App) {
    use lightyear::prelude::client::{InputDelayConfig, InputTimelineConfig};

    // set some input-delay since we are predicting all entities (whatever the fuck that means)
    app.insert_resource(
        InputTimelineConfig::default().with_input_delay(InputDelayConfig::fixed_input_delay(0)),
    );
}