#![feature(variant_count)]

use post_haste::init_postmaster;
use std::process::exit;

use crate::{
    button::ButtonAgent,
    lights::{LightsAgent, LightsMessage},
    sequencer::{SequencerAgent, SequencerMessage},
};

mod consts;
mod button;
mod lights;
mod sequencer;

#[derive(Debug)]
pub(crate) enum Payloads {
    Lights(LightsMessage),
    Sequencer(SequencerMessage),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Addresses {
    LightsAgent,
    ButtonAgent,
    SequencerAgent,
}

init_postmaster!(Addresses, Payloads);

#[tokio::main]
async fn main() {
    println!("Press enter to press the crossing button");

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    postmaster::register_agent!(LightsAgent, LightsAgent, ()).unwrap();
    postmaster::register_agent!(SequencerAgent, SequencerAgent, ()).unwrap();
    postmaster::register_agent!(ButtonAgent, ButtonAgent, ()).unwrap();

    postmaster::send(
        Addresses::SequencerAgent,
        Addresses::SequencerAgent,
        Payloads::Sequencer(SequencerMessage::Begin),
    )
    .await
    .unwrap();

    let _ = tokio::signal::ctrl_c().await;
    println!();
    exit(0);
}
