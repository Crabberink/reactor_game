use std::{net::SocketAddr, time::Duration};

use bevy::prelude::*;
use lightyear::{connection::client::{Client, Connect}, link::Link, netcode::{Key, NetcodeClient, auth::Authentication, client_plugin::NetcodeConfig}, prelude::{LocalAddr, PeerAddr, ReplicationReceiver, UdpIo, client::ClientPlugins}};

use crate::network::protocol::ProtocolPlugin;

pub struct ClientNetworkPlugin;
impl Plugin for ClientNetworkPlugin {
	fn build(&self, app: &mut App) {		
		app.add_plugins(ClientPlugins {
			tick_duration: Duration::from_secs_f64(1.0 / 60.0),
		});
		app.add_systems(Startup, connect_client);
	}
}

#[derive(Resource)]
pub struct LocalClientId(pub u64);

fn connect_client(mut commands: Commands, local_id: Res<LocalClientId>) {
	let server_addr: SocketAddr = "127.0.0.1:3000".parse().expect("Failed to parse server address!");
	let client_addr: SocketAddr = "127.0.0.1:0".parse().expect("Failed to parse client address!");

	let auth = Authentication::Manual {
		server_addr, 
		client_id: local_id.0, 
		private_key: Key::default(), 
		protocol_id: 0, 
	};

	let client = commands.spawn((
		Client,
		LocalAddr(client_addr),
		PeerAddr(server_addr),
		Link::default(),
		ReplicationReceiver,
		NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
		UdpIo::default(),
	)).id();

	commands.trigger(Connect { entity: client});
}