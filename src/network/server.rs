use std::{net::SocketAddr, time::Duration};

use bevy::prelude::*;
use lightyear::{connection::server::Start, netcode::{NetcodeServer, server_plugin::NetcodeConfig}, prelude::{server::{ServerPlugins, ServerUdpIo}, *}};

pub struct ServerNetworkPlugin;
impl Plugin for ServerNetworkPlugin {
	fn build(&self, app: &mut App) {		
		app.add_plugins(ServerPlugins {
			tick_duration: Duration::from_secs_f64(1.0 / 60.0),
		});
		app.add_systems(Startup, start_server);
		app.add_observer(handle_new_client);
	}
}

fn start_server(mut commands: Commands) {
	let server_addr: SocketAddr = "127.0.0.1:3000".parse().expect("Failed to parse server address!");

	let server = commands.spawn((
		NetcodeServer::new(NetcodeConfig::default()),
		LocalAddr(server_addr),
		ServerUdpIo::default(),
	)).id();

	commands.trigger(Start { entity: server});
}

fn handle_new_client(
	trigger: On<Add<LinkOf>>,
	mut commands: Commands,
) {
	commands.entity(trigger.entity).insert(ReplicationSender);
}