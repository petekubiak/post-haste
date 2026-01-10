use post_haste::agent::Agent;

use crate::lights::LightsMessage;
use crate::{Addresses, Payloads, consts, postmaster};

#[derive(Debug)]
pub(crate) enum SequencerMessage {
    Begin,
    ButtonPress,
    #[allow(private_interfaces)]
    InternalMessage(InternalMessage),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TrafficSequenceState {
    Red,
    RedToGreen,
    Green,
    GreenToRed,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PedestrianCrossingSequenceState {
    Stop,
    CrossPending,
    Cross,
    CrossEnding,
}

pub(crate) struct SequencerAgent {
    address: crate::Addresses,

    traffic_light_state: TrafficSequenceState,
    pedestrian_light_state: PedestrianCrossingSequenceState,
}

#[derive(Debug)]
struct InternalMessage {
    traffic_light_state: TrafficSequenceState,
    pedestrian_light_state: Option<PedestrianCrossingSequenceState>,
}

impl Agent for SequencerAgent {
    type Address = crate::Addresses;
    type Message = postmaster::Message;
    type Config = ();

    async fn create(address: Self::Address, _config: Self::Config) -> Self {
        Self {
            address,

            traffic_light_state: TrafficSequenceState::Red,
            pedestrian_light_state: PedestrianCrossingSequenceState::CrossEnding,
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
            SequencerMessage::InternalMessage(internal_message) => {
                self.handle_internal_message(internal_message).await
            }
            SequencerMessage::ButtonPress => self.handle_button_press().await,
        }
    }

    async fn handle_internal_message(&mut self, internal_message: InternalMessage) {
        // Handle traffic light changes
        self.traffic_light_state = internal_message.traffic_light_state;
        postmaster::send(
            crate::Addresses::LightsAgent,
            self.address,
            Payloads::Lights(LightsMessage::SetTrafficLightState(
                self.traffic_light_state.clone(),
            )),
        )
        .await
        .unwrap();

        if let Some(pedestrian_light_state) = internal_message.pedestrian_light_state {
            if self.traffic_light_state == TrafficSequenceState::RedToGreen
                && self.pedestrian_light_state == PedestrianCrossingSequenceState::CrossPending
            {
                // Special case where button has been pressed in the CrossEnding state
                // A delayed message would overwrite the button press
                // If the button has been pressed before getting into the RedToGreen state
                // then do nothing to avoid the button press being overwritten
            } else {
                self.pedestrian_light_state = pedestrian_light_state;
                postmaster::send(
                    crate::Addresses::LightsAgent,
                    self.address,
                    Payloads::Lights(LightsMessage::SetPedestrianLightState(
                        self.pedestrian_light_state.clone(),
                    )),
                )
                .await
                .unwrap();
            }
        }
        self.schedule_next_state().await;
    }

    async fn handle_button_press(&mut self) {
        match self.traffic_light_state {
            TrafficSequenceState::Red => self.handle_button_press_in_red_state().await,
            TrafficSequenceState::RedToGreen | TrafficSequenceState::GreenToRed => {
                self.pedestrian_light_state = PedestrianCrossingSequenceState::CrossPending;
                postmaster::send(
                    Addresses::LightsAgent,
                    self.address,
                    Payloads::Lights(LightsMessage::SetButtonLightState(true)),
                )
                .await
                .unwrap();
                // RedToGreen state already has the next state scheduled
            }
            TrafficSequenceState::Green => {
                self.pedestrian_light_state = PedestrianCrossingSequenceState::CrossPending;
                postmaster::send(
                    Addresses::LightsAgent,
                    self.address,
                    Payloads::Lights(LightsMessage::SetButtonLightState(true)),
                )
                .await
                .unwrap();
                self.schedule_next_state().await;
            }
        }
    }

    async fn handle_button_press_in_red_state(&mut self) {
        match self.pedestrian_light_state {
            PedestrianCrossingSequenceState::Stop => panic!(), // Invalid in red state
            PedestrianCrossingSequenceState::CrossPending
            | PedestrianCrossingSequenceState::Cross => (), // Do nothing as the button is already pressed
            PedestrianCrossingSequenceState::CrossEnding => {
                self.pedestrian_light_state = PedestrianCrossingSequenceState::CrossPending;
                postmaster::send(
                    Addresses::LightsAgent,
                    self.address,
                    Payloads::Lights(LightsMessage::SetButtonLightState(true)),
                )
                .await
                .unwrap();
            }
        }
    }

    async fn begin(&mut self) {
        match self.pedestrian_light_state {
            PedestrianCrossingSequenceState::CrossPending => postmaster::send(
                Addresses::LightsAgent,
                self.address,
                Payloads::Lights(LightsMessage::SetButtonLightState(true)),
            )
            .await
            .unwrap(),
            _ => postmaster::send(
                Addresses::LightsAgent,
                self.address,
                Payloads::Lights(LightsMessage::SetButtonLightState(false)),
            )
            .await
            .unwrap(),
        }
        postmaster::send(
            Addresses::LightsAgent,
            self.address,
            Payloads::Lights(LightsMessage::SetTrafficLightState(
                self.traffic_light_state.clone(),
            )),
        )
        .await
        .unwrap();

        postmaster::send(
            Addresses::LightsAgent,
            self.address,
            Payloads::Lights(LightsMessage::SetPedestrianLightState(
                self.pedestrian_light_state.clone(),
            )),
        )
        .await
        .unwrap();

        self.schedule_next_state().await
    }

    async fn schedule_next_state(&mut self) {
        match self.traffic_light_state {
            TrafficSequenceState::Red => self.calculate_red_state_next_step().await,
            TrafficSequenceState::RedToGreen => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage(InternalMessage {
                    traffic_light_state: TrafficSequenceState::Green,
                    pedestrian_light_state: None,
                })),
            )
            .with_delay(consts::AMBER_TO_GREEN_DELAY)
            .send()
            .await
            .unwrap(),
            TrafficSequenceState::Green => self.calculate_green_state_next_step().await,
            TrafficSequenceState::GreenToRed => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage(InternalMessage {
                    traffic_light_state: TrafficSequenceState::Red,
                    pedestrian_light_state: Some(PedestrianCrossingSequenceState::CrossPending),
                })),
            )
            .with_delay(consts::AMBER_TO_RED_DELAY)
            .send()
            .await
            .unwrap(),
        }
    }

    async fn calculate_red_state_next_step(&self) {
        match self.pedestrian_light_state {
            PedestrianCrossingSequenceState::Stop => panic!(),
            PedestrianCrossingSequenceState::CrossPending => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage(InternalMessage {
                    traffic_light_state: TrafficSequenceState::Red,
                    pedestrian_light_state: Some(PedestrianCrossingSequenceState::Cross),
                })),
            )
            .with_delay(consts::CROSSING_START_DELAY)
            .send()
            .await
            .unwrap(),

            PedestrianCrossingSequenceState::Cross => {
                postmaster::message(
                    self.address,
                    self.address,
                    Payloads::Sequencer(SequencerMessage::InternalMessage(InternalMessage {
                        traffic_light_state: TrafficSequenceState::Red,
                        pedestrian_light_state: Some(PedestrianCrossingSequenceState::CrossEnding),
                    })),
                )
                .with_delay(consts::CROSSING_LENGTH)
                .send()
                .await
                .unwrap();
                // Turn off the button light
                postmaster::send(
                    Addresses::LightsAgent,
                    self.address,
                    Payloads::Lights(LightsMessage::SetButtonLightState(false)),
                )
                .await
                .unwrap()
            }
            PedestrianCrossingSequenceState::CrossEnding => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage(InternalMessage {
                    traffic_light_state: TrafficSequenceState::RedToGreen,
                    pedestrian_light_state: Some(PedestrianCrossingSequenceState::Stop),
                })),
            )
            .with_delay(consts::CROSSING_END_DELAY)
            .send()
            .await
            .unwrap(),
        }
    }

    async fn calculate_green_state_next_step(&self) {
        match self.pedestrian_light_state {
            PedestrianCrossingSequenceState::Stop => (), // Do nothing, light stays green
            PedestrianCrossingSequenceState::CrossPending => postmaster::message(
                self.address,
                self.address,
                Payloads::Sequencer(SequencerMessage::InternalMessage(InternalMessage {
                    traffic_light_state: TrafficSequenceState::GreenToRed,
                    pedestrian_light_state: Some(PedestrianCrossingSequenceState::CrossPending),
                })),
            )
            .with_delay(consts::GREEN_TO_AMBER_DELAY)
            .send()
            .await
            .unwrap(),
            PedestrianCrossingSequenceState::Cross
            | PedestrianCrossingSequenceState::CrossEnding => panic!(), // Invalid in green states
        }
    }
}
