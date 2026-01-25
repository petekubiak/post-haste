use post_haste::agent::Agent;

use crate::lights::LightsMessage;
use crate::{Addresses, Payloads, consts, postmaster};

#[derive(Debug)]
pub(crate) enum SequencerMessage {
    Begin,
    ButtonPress,
    #[allow(private_interfaces)]
    InternalMessage,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SequencerState {
    Green,
    GreenCrossPending,
    GreenToRed,
    RedCrossPending,
    RedCrossing,
    RedCrossEnding,
    RedToGreen,
    RedToGreenCrossPending,
}

pub(crate) struct SequencerAgent {
    address: crate::Addresses,
    state: SequencerState,
}

impl Agent for SequencerAgent {
    type Address = crate::Addresses;
    type Message = postmaster::Message;
    type Config = ();

    async fn create(address: Self::Address, _config: Self::Config) -> Self {
        Self {
            address,
            state: SequencerState::RedCrossEnding,
        }
    }

    async fn run(mut self, mut inbox: post_haste::agent::Inbox<Self::Message>) -> ! {
        loop {
            let received_message = inbox.recv().await.unwrap();
            match received_message.payload {
                Payloads::Sequencer(message) => self.handle_message(message).await,
                _ => println!(
                    "SequencerAgent received unsupported message {:?}",
                    received_message.payload
                ),
            }
        }
    }
}

impl SequencerAgent {
    async fn handle_message(&mut self, message: SequencerMessage) {
        match message {
            SequencerMessage::Begin => self.begin().await,
            SequencerMessage::InternalMessage => self.handle_internal_message().await,
            SequencerMessage::ButtonPress => self.handle_button_press().await,
        }
    }

    async fn send_current_state_to_lights_agent(&mut self) {
        postmaster::send(
            Addresses::LightsAgent,
            self.address,
            Payloads::Lights(LightsMessage::SetSequenceState {
                sequence_state: self.state.clone(),
            }),
        )
        .await
        .unwrap();
    }

    async fn handle_internal_message(&mut self) {
        self.set_next_state().await;
        self.send_current_state_to_lights_agent().await;
    }

    async fn handle_button_press(&mut self) {
        match self.state {
            SequencerState::Green => {
                self.state = SequencerState::GreenCrossPending;
                self.send_current_state_to_lights_agent().await;
                self.schedule_next_state().await;
            }
            SequencerState::GreenCrossPending
            | SequencerState::GreenToRed
            | SequencerState::RedCrossPending
            | SequencerState::RedCrossing
            | SequencerState::RedCrossEnding
            | SequencerState::RedToGreenCrossPending => (),
            SequencerState::RedToGreen => {
                self.state = SequencerState::RedToGreenCrossPending;
                self.send_current_state_to_lights_agent().await;
                // Do not schedule next state - in RedToGreen state, a delayed
                // message will already have been sent!
            }
        }
    }

    async fn begin(&mut self) {
        self.send_current_state_to_lights_agent().await;
        self.schedule_next_state().await;
    }

    async fn set_next_state(&mut self) {
        match self.state {
            SequencerState::Green => unreachable!(),
            SequencerState::GreenCrossPending => {
                self.state = SequencerState::GreenToRed;
                self.schedule_next_state().await;
            }
            SequencerState::GreenToRed => {
                self.state = SequencerState::RedCrossPending;
                self.schedule_next_state().await;
            }
            SequencerState::RedCrossPending => {
                self.state = SequencerState::RedCrossing;
                self.schedule_next_state().await;
            }
            SequencerState::RedCrossing => {
                self.state = SequencerState::RedCrossEnding;
                self.schedule_next_state().await;
            }
            SequencerState::RedCrossEnding => {
                self.state = SequencerState::RedToGreen;
                self.schedule_next_state().await;
            }
            SequencerState::RedToGreen => {
                self.state = SequencerState::Green;
            }
            SequencerState::RedToGreenCrossPending => {
                self.state = SequencerState::GreenCrossPending;
                self.schedule_next_state().await;
            }
        }
        self.send_current_state_to_lights_agent().await;
    }

    async fn schedule_next_state(&mut self) {
        match self.state {
            SequencerState::Green => unreachable!(),
            SequencerState::GreenCrossPending => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage),
            )
            .with_delay(consts::GREEN_TO_AMBER_DELAY)
            .send()
            .await
            .unwrap(),
            SequencerState::GreenToRed => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage),
            )
            .with_delay(consts::AMBER_TO_RED_DELAY)
            .send()
            .await
            .unwrap(),
            SequencerState::RedCrossPending => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage),
            )
            .with_delay(consts::CROSSING_START_DELAY)
            .send()
            .await
            .unwrap(),
            SequencerState::RedCrossing => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage),
            )
            .with_delay(consts::CROSSING_LENGTH)
            .send()
            .await
            .unwrap(),
            SequencerState::RedCrossEnding => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage),
            )
            .with_delay(consts::CROSSING_END_DELAY)
            .send()
            .await
            .unwrap(),
            SequencerState::RedToGreen | SequencerState::RedToGreenCrossPending => {
                postmaster::message(
                    self.address,
                    self.address,
                    Payloads::Sequencer(SequencerMessage::InternalMessage),
                )
                .with_delay(consts::AMBER_TO_GREEN_DELAY)
                .send()
                .await
                .unwrap()
            }
        }
    }
}
