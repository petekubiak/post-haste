#![no_std]
#![feature(variant_count)]

use embassy_executor::Spawner;
use post_haste::init_postmaster;

use crate::polite_agent::PoliteAgent;

pub mod polite_agent;

enum Payloads {
    Hello,
}

#[derive(Clone, Copy, Debug)]
enum Address {
    PoliteAgentA,
    PoliteAgentB,
}

init_postmaster!(Address, Payloads);

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
