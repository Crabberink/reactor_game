use crate::LoadingState;
use crate::network::client::{ClientTransports, GameClient};
use crate::network::server::{GameServer, ServerTransports};
use crate::network::shared::{CLIENT_PORT, SERVER_ADDR, SERVER_PORT, SHARED_SETTINGS};
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::{Level, LogPlugin};
use bevy::mesh::MeshPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use clap::{Parser, Subcommand};
use lightyear::prelude::*;
use std::time::Duration;

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Option<Mode>,
}

impl Cli {
    // pub fn client_id(&self) -> Option<u64> {
    //     match &self.mode {
    //         Some(Mode::Client { client_id }) => *client_id,
    //         Some(Mode::HostClient { client_id }) => *client_id,
    //         _ => None,
    //     }
    // }

    pub fn headless(&self) -> bool {
        matches!(self.mode.as_ref(), Some(Mode::Server))
    }

    pub fn create_app(add_inspector: bool, _mode: Option<&Mode>, headless: bool) -> App {
        if !headless { return new_gui_app(add_inspector); }

        // let loop_wait = match mode {
        //     Some(Mode::Client { .. }) => { Some(Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ)) },
        //     Some(Mode::HostClient { .. }) => { Some(Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ)) },
        //     _ => None
        // };

        new_headless_app(None)
    }

    pub fn build_app(&self, tick_duration: Duration, add_inspector: bool) -> App {
        let mut app = Cli::create_app(add_inspector, self.mode.as_ref(), self.headless());

        match self.mode {
            Some(Mode::Client { .. }) => {
                // steam resources go here
                app.add_plugins(lightyear::prelude::client::ClientPlugins { tick_duration });

                app
            },
            Some(Mode::Server) => {
                // steam resources go here
                app.add_plugins(lightyear::prelude::server::ServerPlugins { tick_duration });

                app
            },
            Some(Mode::HostClient { .. }) => {
                // steam resources go here
                app.add_plugins(lightyear::prelude::client::ClientPlugins { tick_duration });
                app.add_plugins(lightyear::prelude::server::ServerPlugins { tick_duration });

                app
            },
            None => {
                panic!(
                    "Mode is required. Possible options: {}",
                    "Initiating protocol: Fuck You. I am not implementing this shit, figure it out",
                );
            }
        }
    }

    pub fn spawn_connections(&self, app: &mut App) {
        // let conditioner = Some(LinkConditionerConfig::average_condition().half());
        let conditioner = None;
        
        match self.mode {
            Some(Mode::Client { client_id }) => {
                let _client = app
                    .world_mut()
                    .spawn(GameClient {
                        client_id: client_id.expect("Client id needs to be specified with -c ID"),
                        client_port: CLIENT_PORT,
                        server_addr: SERVER_ADDR,
                        conditioner: conditioner.clone().map(RecvLinkConditioner::new),
                        transport: ClientTransports::Udp,
                        shared_settings: SHARED_SETTINGS,
                    }).id();
                app.add_systems(OnEnter(LoadingState::Loaded), crate::network::client::connect);
            },
            Some(Mode::Server) => {
                let _server = app
                    .world_mut()
                    .spawn(GameServer {
                        conditioner: conditioner.clone().map(RecvLinkConditioner::new),
                        transport: ServerTransports::Udp { local_port: SERVER_PORT },
                        shared_settings: SHARED_SETTINGS,
                    }).id();

                app.add_systems(OnEnter(LoadingState::Loaded), crate::network::server::start);
            },
            Some(Mode::HostClient { .. }) => {
                let server = app
                    .world_mut()
                    .spawn(GameServer {
                        conditioner: conditioner.clone().map(RecvLinkConditioner::new),
                        transport: ServerTransports::Udp { local_port: SERVER_PORT },
                        shared_settings: SHARED_SETTINGS,
                    }).id();

                let _client = app.
                    world_mut()
                    .spawn((Client, Name::new("HostClient"), LinkOf { endpoint: server }))
                    .id();

                app.add_systems(OnEnter(LoadingState::Loaded), (
                    crate::network::server::start,
                    crate::network::client::connect
                ).chain());
            },
            None => {

            }
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    Client {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
    Server,
    HostClient {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
}

pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "Reactor game".to_string(),
            resolution: (960, 540).into(),
            present_mode: PresentMode::AutoVsync,
            prevent_default_event_handling: true,
            ..Default::default()
        }),
        ..default()
    }
}

pub fn log_plugin() -> LogPlugin {
    let plugin = LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_enhanced_input::action::fns=error".to_string(),
        ..default()
    };
    // let plugin = LogPlugin {
    //     custom_layer: lightyear_debug_custom_layer,
    //     fmt_layer: non_lightyear_debug_fmt_layer,
    //     ..plugin
    // };
    plugin
}

pub fn new_gui_app(add_inspector: bool) -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                // https://github.com/bevyengine/bevy/issues/10157
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .set(window_plugin())
            .set(ImagePlugin::default_nearest()),
    );
    // we want the same frequency of updates for both focused and unfocused
    // Otherwise when testing the movement can look choppy for unfocused windows
    app.insert_resource(WinitSettings::continuous());

    if add_inspector {
        app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    }
    app
}

pub fn new_headless_app(loop_wait: Option<Duration>) -> App {
    let mut app = App::new();
    let minimal_plugins = match loop_wait {
        Some(wait) => MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(wait)),
        None => MinimalPlugins.build(),
    };
    app.add_plugins((
        minimal_plugins,
        TransformPlugin,
        bevy::input::InputPlugin,
        log_plugin(),
        StatesPlugin,
        DiagnosticsPlugin,
        AssetPlugin::default(),
        MeshPlugin,
    ));
    app
}