use std::{net::SocketAddr, time::Duration};
use std::net::Ipv4Addr;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use lightyear::{connection::client::{Connect}, link::Link, netcode::{Key, NetcodeClient, auth::Authentication, client_plugin::NetcodeConfig}, prelude::{LocalAddr, PeerAddr, ReplicationReceiver, UdpIo, client::ClientPlugins}};
use lightyear::netcode::client::ClientConfig;
use lightyear::prelude::*;
// use lightyear::prelude::client::{ConnectTarget, SteamClientIo};
use serde::{Deserialize, Serialize};
use crate::network::shared::SharedSettings;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientTransports {
	Udp,
	// Steam,
}


#[derive(Component, Clone, Debug)]
#[component(on_add = GameClient::on_add)]
pub struct GameClient {
	pub client_id: u64,
	pub client_port: u16,
	pub server_addr: SocketAddr,
	pub conditioner: Option<RecvLinkConditioner>,
	pub transport: ClientTransports,
	pub shared_settings: SharedSettings,
}

impl GameClient {
	fn on_add(mut world: DeferredWorld, context: HookContext) {
		let entity = context.entity;

		world.commands().queue(move |world: &mut World| -> Result {
			world.insert_resource(PredictionManager::default());

			let mut entity_mut = world.entity_mut(entity);
			let settings = entity_mut.take::<GameClient>().unwrap();
			let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client_port);

			entity_mut.insert((
				Client,
				ReplicationReceiver,
				Link::default().with_conditioner(settings.conditioner.clone()),
				LocalAddr(client_addr),
				PeerAddr(settings.server_addr),
				Name::from("Client"),
			));

			let add_netcode = |entity_mut: &mut EntityWorldMut| -> Result {
				let auth = Authentication::Manual {
					server_addr: settings.server_addr,
					client_id: settings.client_id,
					private_key: settings.shared_settings.private_key,
					protocol_id: settings.shared_settings.protocol_id,
				};

				let netcode_config = NetcodeConfig {
					client_timeout_secs: 5,
					token_expire_secs: -1,
					..default()
				};

				entity_mut.insert(NetcodeClient::new(auth, netcode_config)?);

				Ok(())
			};

			match settings.transport {
				ClientTransports::Udp => {
					add_netcode(&mut entity_mut)?;
					entity_mut.insert(UdpIo::default());
				},
				// ClientTransports::Steam => {
				// 	entity_mut.insert(SteamClientIo {
				// 		target: ConnectTarget::Addr(settings.server_addr),
				// 		config: Default::default(),
				//   	});
				// },
			};

			Ok(())
		});
	}
}

pub(crate) fn connect(mut commands: Commands, client: Single<Entity, With<Client>>) {
	commands.trigger(Connect { entity: client.into_inner() });
}

// fn connect_client(mut commands: Commands, local_id: Res<LocalClientId>) {
// 	let server_addr: SocketAddr = "127.0.0.1:3000".parse().expect("Failed to parse server address!");
// 	let client_addr: SocketAddr = "127.0.0.1:0".parse().expect("Failed to parse client address!");
//
// 	let test = Client
//
// 	let auth = Authentication::Manual {
// 		server_addr,
// 		client_id: local_id.0,
// 		private_key: Key::default(),
// 		protocol_id: 0,
// 	};
//
// 	let client = commands.spawn((
// 		LocalAddr(client_addr),
// 		PeerAddr(server_addr),
// 		Link::default(),
// 		ReplicationReceiver,
// 		NetcodeClient::new(auth, NetcodeConfig::default()).unwrap(),
// 		UdpIo::default(),
// 	)).id();
//
// 	commands.trigger(Connect { entity: client});
// }