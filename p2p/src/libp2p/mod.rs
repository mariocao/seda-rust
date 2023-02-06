mod behaviour;
mod transport;

#[cfg(test)]
mod libp2p_test;

use async_std::io::{self, prelude::BufReadExt};
use libp2p::{
    futures::StreamExt,
    gossipsub::{GossipsubEvent, IdentTopic},
    identity::{self},
    mdns::Event as MdnsEvent,
    swarm::{Swarm, SwarmEvent},
    Multiaddr,
    PeerId,
};
use seda_runtime_sdk::p2p::{P2PCommand, P2PMessage};
use tokio::sync::mpsc::{Receiver, Sender};

use self::behaviour::SedaBehaviour;
use crate::{
    errors::Result,
    libp2p::{behaviour::SedaBehaviourEvent, transport::build_tcp_transport},
};

pub const GOSSIP_TOPIC: &str = "testnet";

pub struct P2PServer {
    pub known_peers:              Vec<String>,
    pub local_key:                identity::Keypair,
    pub server_address:           String,
    pub swarm:                    Swarm<SedaBehaviour>,
    pub message_sender_channel:   Sender<P2PMessage>,
    pub command_receiver_channel: Receiver<P2PCommand>,
}

impl P2PServer {
    pub async fn start_from_config(
        server_address: &str,
        known_peers: &[String],
        message_sender_channel: Sender<P2PMessage>,
        command_receiver_channel: Receiver<P2PCommand>,
    ) -> Result<Self> {
        // Generate Peer ID
        // TODO: Support peer id from config and storage
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        tracing::info!("Local peer id: {:?}", local_peer_id);

        // Create transport and behaviour
        let transport = build_tcp_transport(local_key.clone())?;
        let seda_behaviour = SedaBehaviour::new(&local_key).await?;

        let mut swarm = Swarm::with_threadpool_executor(transport, seda_behaviour, PeerId::from(local_key.public()));
        swarm.listen_on(server_address.parse()?)?;

        // Create channels so we can listen for p2p messages

        Ok(Self {
            known_peers: known_peers.to_vec(),
            local_key,
            server_address: server_address.to_string(),
            swarm,
            message_sender_channel,
            command_receiver_channel,
        })
    }

    pub async fn dial_peers(&mut self) -> Result<()> {
        self.known_peers.iter().for_each(|peer_addr| {
            if let Ok(remote) = peer_addr.parse::<Multiaddr>() {
                match self.swarm.dial(remote) {
                    Ok(_) => {
                        tracing::debug!("Dialed {}", peer_addr);
                    }
                    Err(error) => tracing::warn!("Couldn't dial peer ({}): {:?}", peer_addr, error),
                };
            } else {
                tracing::warn!("Couldn't dial peer with address: {}", peer_addr);
            }
        });

        Ok(())
    }

    pub async fn loop_stream(&mut self) -> Result<()> {
        // TODO: Remove stdin feature
        let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
        let topic = IdentTopic::new(GOSSIP_TOPIC);

        loop {
            tokio::select! {
                // TODO: Remove stdin feature
                line = stdin.select_next_some() => {
                    if let Err(e) = self.swarm
                        .behaviour_mut().gossipsub
                        .publish(topic.clone(), line.expect("Stdin not to close").as_bytes()) {
                        tracing::error!("Publish error: {e:?}");
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => tracing::info!("Listening on {:?}", address),
                    SwarmEvent::Behaviour(SedaBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            tracing::debug!("mDNS discovered a new peer: {}", peer_id);
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(SedaBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            tracing::debug!("mDNS discover peer has expired: {}", peer_id);
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    }
                    SwarmEvent::Behaviour(SedaBehaviourEvent::Gossipsub(GossipsubEvent::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => {
                        tracing::info!(
                            "Got message: '{}' with id: {id} from peer: {peer_id}",
                            String::from_utf8_lossy(&message.data),
                        );

                        let source: Option<String> = message.source.map(|peer| peer.to_string());

                        if let Err(err) = self.message_sender_channel.send(P2PMessage { source, data: message.data }).await {
                            tracing::error!("Couldn't send message through channel: {err}");
                        }
                    },
                    _ => {}
                },
                task = self.command_receiver_channel.recv() => match task {
                    Some(P2PCommand::Broadcast(data)) => {
                        if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(topic.clone(), data) {
                            tracing::error!("Publish error: {e:?}");
                        }
                    },
                    Some(P2PCommand::Unicast(_unicast)) => {
                        unimplemented!("Todo unicast");
                    },
                    None => {}
                },
            }
        }
    }
}
