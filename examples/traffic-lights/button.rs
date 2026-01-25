use tokio::io::{self, AsyncBufReadExt, BufReader};

use crate::{Addresses, Payloads, lights::LightsMessage, postmaster, sequencer::SequencerMessage};

pub async fn button_task() -> ! {
    let mut reader = BufReader::new(io::stdin()).lines();
    loop {
        if let Some(_) = reader.next_line().await.unwrap() {
            postmaster::send(
                Addresses::SequencerAgent,
                Addresses::ButtonTask,
                Payloads::Sequencer(SequencerMessage::ButtonPress),
            )
            .await
            .unwrap();

            // Optionally, also send a message which explains what message has been send
            postmaster::send(
                Addresses::LightsAgent,
                Addresses::ButtonTask,
                Payloads::Lights(LightsMessage::DebugMessage(String::from(
                    "Message sent from ButtonTask to LightsAgent",
                ))),
            )
            .await
            .unwrap();
        }
    }
}
