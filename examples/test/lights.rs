use post_haste::agent::{Agent, Inbox};

use crate::{Addresses, Payloads, postmaster, sequencer};

#[derive(Debug)]
pub(crate) enum LightsMessage {
    SetTrafficLightState(sequence::TrafficSequenceState),
    SetPedestrianLightState(sequencer::PedestrianCrossingSequenceState),
    SetButtonLightState(bool),
    Display,
}

struct TrafficLights {
    red: bool,
    amber: bool,
    green: bool,
}

impl Default for TrafficLights {
    fn default() -> Self {
        Self {
            red: true,
            amber: false,
            green: false,
        }
    }
}

impl From<sequencer::TrafficSequenceState> for TrafficLights {
    fn from(value: sequencer::TrafficSequenceState) -> Self {
        match value {
            sequencer::TrafficSequenceState::Red => TrafficLights {
                red: true,
                amber: false,
                green: false,
            },
            sequencer::TrafficSequenceState::RedToGreen => TrafficLights {
                red: true,
                amber: true,
                green: false,
            }
            sequencer::TrafficSequenceState::Green => TrafficLights {
                red: false,
                amber: false,
                green: true,
            }
            sequencer::TrafficSequenceState::GreenToRed => TrafficLights {
                red: false,
                amber: true,
                green: false,
            }
        }
    }
}

struct PedestrianLights {
    stop: bool,
    cross: bool,
}

impl Default for PedestrianLights {
    fn default() -> Self {
        Self {
            stop: true,
            cross: false,
        }
    }
}

impl From<sequencer::PedestrianCrossingSequenceState> for PedestrianLights {
    fn from(value: sequencer::PedestrianCrossingSequenceState) -> Self {
        match value {
            sequencer::PedestrianCrossingSequenceState::Cross => PedestrianLights {
                stop: false,
                cross: true,
            },
            sequencer::PedestrianCrossingSequenceState::Stop
            | sequencer::PedestrianCrossingSequenceState::CrossEnding
            | sequencer::PedestrianCrossingSequenceState::CrossPending => {
                PedestrianLights {
                    stop: true,
                    cross: false,
                }
            }
        }
    }
}

pub(crate) struct LightsAgent {
    _address: Addresses,

    traffic_light_state: TrafficLights,
    pedestrian_light_state: PedestrianLights,
    cross_pending: bool,
}

impl Agent for LightsAgent {
    type Address = Addresses;
    type Message = postmaster::Message;
    type Config = ();

    async fn create(address: Self::Address, _config: Self::Config) -> Self {
        Self {
            _address: address,
            traffic_light_state: TrafficLights{
                ..Default::default()
            },
            pedestrian_light_state: PedestrianLights {
                ..Default::default()
            }
            cross_pending: false,
        }
    }

    async fn run(mut self, mut inbox: Inbox<Self::Message>) -> ! {
        loop {
            let optional_message = inbox.recv().await;
            if let Some(message) = optional_message {
                self.message_handler(message.payload);
            }
        }
    }
}

impl LightsAgent {
    fn message_handler(&mut self, message: Payloads) {
        if let Payloads::Lights(lights_message) = message {
            match lights_message {
                LightsMessage::SetTrafficLightState(traffic_light_sequencer_state) => {
                    self.traffic_light_state = 
                        TrafficLights::from(traffic_light_sequencer_state)
                },
                LightsMessage::SetPedestrianLightState(pedestrian_crossing_sequencer_state) => {
                    self.pedestrian_light_state =
                        TrafficLights::from(pedestrian_crossing_sequencer_state)
                },
                LightsMessage::SetButtonLightState(cross_pending) => {
                    self.cross_pending = cross_pending
                }
                LightsMessage::Display => self.display_ascii()
            }
        }
    }

    fn display_ascii(&self) {
        // ----
        // |⚫|   ----
        // ----   |🖐️|
        // |⚫|   |🏃‍♂️‍➡️|
        // ----   ----
        // |🟢|
        // ----   |🔴|

        let mut text = String::new();

        text.push_str("----\n");

        if self.traffic_light_state.red {
            text.push_str("|🔴|");
        } else {
            text.push_str("|⚫|");
        }

        text.push_str("   ----\n");

        text.push_str("----   ");

        if self.pedestrian_light_state.stop {
            text.push_str("|🖐️|\n");
        } else {
            text.push_str("|  |\n");
        }

        text.push_str("   ");

        if self.traffic_light_state.amber {
            text.push_str("|🟡|");
        } else {
            text.push_str("|⚫|");
        }

        text.push_str("   ");

        if self.pedestrian_light_state.cross {
            text.push_str("|🏃‍♂️‍➡️|\n");
        } else {
            text.push_str("|  |\n");
        }

        text.push_str("----   ----\n");

        if self.traffic_light_state.green {
            text.push_str("|🟢|");
        } else {
            text.push_str("|⚫|");
        }

        for _ in 0..30 {
            println!();
        }

        println!("{}", text);
    }
}