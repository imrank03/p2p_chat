//! copy from Ping example
//! use msg protocol
//! See ../src/tutorial.rs for a step-by-step guide building the example below.
//!
//! In the first terminal window, run:
//!
//! ```sh
//! cargo run --example p2p_chat
//! ```
//!
//! It will print the PeerId and the listening addresses, e.g. `Listening on
//! "/ip4/0.0.0.0/tcp/24915"`
//!
//! In the second terminal window, start a new instance of the example with:
//!
//! ```sh
//! cargo run --example p2p_chat  -- /ip4/127.0.0.1/tcp/24915
//! ```
//!
//! The two nodes establish a connection, negotiate the ping protocol
//! and begin pinging each other.

use futures::{prelude::*, select};

use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{identity, Multiaddr, PeerId};
use std::error::Error;

use async_std::io;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = libp2p::development_transport(local_key).await?;

    // Create a ping network behaviour.
    //
    // For illustrative purposes, the ping protocol is configured to
    // keep the connection alive, so a continuous sequence of pings
    // can be observed.
    let behaviour = p2p_chat::Behaviour::new();

    let mut swarm = Swarm::new(transport, behaviour, local_peer_id);

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Dial the peer identified by the multi-address given as the second
    // command-line argument, if any.
    if let Some(addr) = std::env::args().nth(1) {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Dialed {}", addr)
    }

    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    loop {
        select! {
            line = stdin.select_next_some() => swarm
                .behaviour_mut()
                .send(line.expect("Stdin not to close").as_bytes()),

            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
                SwarmEvent::Behaviour(event) => {
                    match event {
                        p2p_chat::Event{peer,result} =>{
                            match result {
                                p2p_chat::MsgContent{data}=>{
                                    println!("recv:from->{:?}\n {:?}",peer,std::str::from_utf8(&data).unwrap());
                                }
                            }
                        }
                    };
                },
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("New connect {:?}", peer_id);
                    swarm.behaviour_mut()
                    .insert(&peer_id);

                    let peers = swarm.connected_peers();
                    for p in peers {
                        println!("peer {}",p);
                    }
                },
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    println!("disconnect {:?}", peer_id);
                    swarm.behaviour_mut()
                    .remove(&peer_id);
                },
                //default
                _ => {}
            }
        } // select
    }
}
