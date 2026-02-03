//! The "button" task, which asynchronously awaits for the user to press enter,
//! signalling that the pedestrian wants to cross

use tokio::io::{self, AsyncBufReadExt, BufReader};

use crate::{
    Addresses, Payloads, display::DisplayMessage, postmaster, sequencer::SequencerMessage,
};

/// Asynchronous task for checking the crossing "button"
pub async fn button_task() -> ! {
    let mut reader = BufReader::new(io::stdin()).lines();
    loop {
        // Infinitely asynchronously await for a line of input text from the terminal
        if let Some(_) = reader.next_line().await.unwrap() {
            // Upon receiving a line of text, send a message to the Sequencer Agent
            postmaster::send(
                Addresses::SequencerAgent,
                Addresses::ButtonTask,
                Payloads::Sequencer(SequencerMessage::ButtonPress),
            )
            .await
            .unwrap();

            // Also send a debug message - this is just to help understand how this example works
            postmaster::send(
                Addresses::DisplayAgent,
                Addresses::ButtonTask,
                Payloads::Display(DisplayMessage::DebugMessage(String::from(
                    "Message sent from ButtonTask to LightsAgent",
                ))),
            )
            .await
            .unwrap();
        }
    }
}
