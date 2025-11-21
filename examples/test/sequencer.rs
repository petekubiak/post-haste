use post_haste::agent::Agent;

use crate::lights::LightsMessage;
use crate::{Addresses, Payloads, consts, postmaster};

#[derive(Debug)]
pub(crate) enum SequencerMessage {
    Begin,
    ButtonPress,
    #[allow(private_interfaces)],
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
                Payloads::SequencerMessage(message) => self.handle_message(message).await,
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
            crate::Addresses::LightsMessage,
            self.address,
            Payloads::Lights(LightsMessage::SetTrafficLightsState(
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
                    crate::Addresses::LightAgent,
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

    async fn handle_button_press
}

