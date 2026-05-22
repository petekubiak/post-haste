#![no_std]

use embassy_executor::Spawner;
use post_haste::init_postmaster;

use crate::polite_agent::PoliteAgent;

pub mod polite_agent;

#[post_haste::payloads]
enum Payloads {
    Hello,
}

#[derive(Clone, Copy, Debug)]
#[post_haste::addresses]
enum Address {
    PoliteAgentA,
    PoliteAgentB,
}

init_postmaster!();

pub async fn run(spawner: Spawner) {
    postmaster::register_agent!(spawner, PoliteAgentA, PoliteAgent, ()).unwrap();

    postmaster::register_agent!(spawner, PoliteAgentB, PoliteAgent, ()).unwrap();

    postmaster::send(
        Address::PoliteAgentA,
        Address::PoliteAgentB,
        Payloads::Hello,
    )
    .await
    .unwrap();
}
