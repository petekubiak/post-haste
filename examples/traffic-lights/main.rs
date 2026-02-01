// variant_count is required for init_postmaster
#![feature(variant_count)]

use post_haste::init_postmaster;
use std::process::exit;

use crate::{
    button::button_task,
    display::{DisplayAgent, DisplayMessage},
    sequencer::{SequencerAgent, SequencerMessage},
};

mod button;
mod consts;
mod display;
mod sequencer;

// Declare the payloads for project. These are the messages that each agent can
// send. The contents of each message is usually an enum which is defined in
// the file implementing that agent
#[derive(Debug)]
pub(crate) enum Payloads {
    // Messages to be sent to the display agent
    Display(DisplayMessage),
    // Messages to be sent to the sequencer agent
    Sequencer(SequencerMessage),
}

// Declare the addresses for post-haste. Messages can be sent from any address
// but Main and ButtonTask do not have agent implementations, so cannot receive
// messages
#[derive(Debug, Clone, Copy)]
pub(crate) enum Addresses {
    Main,
    DisplayAgent,
    SequencerAgent,
    ButtonTask,
}

init_postmaster!(Addresses, Payloads);

#[tokio::main]
async fn main() {
    // Register each agent with it's address and the struct implementing the agent
    // The config is not used for this project
    postmaster::register_agent!(DisplayAgent, DisplayAgent, ()).unwrap();
    postmaster::register_agent!(SequencerAgent, SequencerAgent, ()).unwrap();
    // Spawn the button task using tokio
    tokio::spawn(button_task());

    // Send a message to the Display Agent with a payload requesting for a message
    // to be displayed in the terminal.
    postmaster::send(
        Addresses::DisplayAgent,
        Addresses::Main,
        Payloads::Display(DisplayMessage::DebugMessage(
            "Press enter to press the crossing button".to_string(),
        )),
    )
    .await
    .unwrap();

    // Send a message to the Sequencer Agent requesting for the sequencing to begin
    postmaster::send(
        Addresses::SequencerAgent,
        Addresses::Main,
        Payloads::Sequencer(SequencerMessage::Begin),
    )
    .await
    .unwrap();

    // If CTRL+C is pressed then the program will exit nicely
    let _ = tokio::signal::ctrl_c().await;
    println!();
    exit(0);
}
