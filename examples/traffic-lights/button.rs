use post_haste::agent::Agent;
use tokio::io::{self, AsyncBufReadExt, BufReader, Lines, Stdin};

use crate::{Addresses, Payloads, lights::LightsMessage, postmaster, sequencer::SequencerMessage};

pub(crate) struct ButtonAgent {
    address: Addresses,
    reader: Lines<BufReader<Stdin>>,
}

impl Agent for ButtonAgent {
    type Address = Addresses;
    type Message = postmaster::Message;
    type Config = ();

    async fn create(address: Self::Address, _config: Self::Config) -> Self {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin).lines();

        Self {
            address,
            reader,
        }
    }

    async fn run(mut self, _inbox: post_haste::agent::Inbox<Self::Message>) -> ! {
        loop {
            if let Some(_) = self.reader.next_line().await.unwrap() {
                postmaster::send(
                    Addresses::SequencerAgent,
                    self.address,
                    Payloads::Sequencer(SequencerMessage::ButtonPress),
                )
                .await
                .unwrap();

                postmaster::send(
                    Addresses::LightsAgent,
                    self.address,
                    Payloads::Lights(LightsMessage::Display),
                )
                .await
                .unwrap();
            }
        }
    }
}