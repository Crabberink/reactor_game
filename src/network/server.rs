use crate::network::shared::SharedSettings;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use lightyear::{connection::server::Start, netcode::{NetcodeServer, server_plugin::NetcodeConfig}, prelude::{server::ServerUdpIo, *}};
// use lightyear::prelude::server::{ListenTarget, SteamEndpoint};
// use lightyear::prelude::SessionConfig;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::net::SocketAddr;

pub struct ServerNetworkPlugin;
impl Plugin for ServerNetworkPlugin {
	fn build(&self, app: &mut App) {
		app.add_observer(handle_new_client);
	}
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerTransports {
	Udp { local_port: u16 },
	// Steam { local_port: u16 },
}

#[derive(Component, Debug)]
#[component(on_add = GameServer::on_add)]
pub struct GameServer {
	pub conditioner: Option<RecvLinkConditioner>,
	pub transport: ServerTransports,
	pub shared_settings: SharedSettings,
}

impl GameServer {
	fn on_add(mut world: DeferredWorld, context: HookContext) {
		let entity = context.entity;

		world.commands().queue(move |world: &mut World| -> Result {
			let mut entity_mut = world.entity_mut(entity);

			let settings = entity_mut.take::<GameServer>().unwrap();
			entity_mut.insert((
				Name::from("Server"),
				Endpoint::new(settings.conditioner.clone())
			));

			let add_netcode = |entity_mut: &mut EntityWorldMut| {
				let private_key = settings.shared_settings.private_key;

				entity_mut.insert(NetcodeServer::new(NetcodeConfig {
					protocol_id: settings.shared_settings.protocol_id,
					private_key,
					..Default::default()
				}));
			};

			match settings.transport {
				ServerTransports::Udp { local_port } => {
					add_netcode(&mut entity_mut);
					let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
					entity_mut.insert((LocalAddr(server_addr), ServerUdpIo));
				},
				// ServerTransports::Steam { local_port } => {
				// 	let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
				// 	entity_mut.insert((
				// 		SteamEndpoint {
				// 			target: ListenTarget::Addr(server_addr),
				// 			config: SessionConfig::default(),
				// 		},
				// 		Server,
				// 	));
				// }
			};
			Ok(())
		});
	}
}

pub(crate) fn start(mut commands: Commands, server: Single<Entity, With<Server>>) {
	commands.trigger(Start { entity: server.into_inner() });
}

fn handle_new_client(
	trigger: On<Add<LinkOf>>,
	mut commands: Commands,
) {
	commands.entity(trigger.entity).insert(ReplicationSender);
}