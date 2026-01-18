use crossterm::{execute, style::Stylize, terminal};
use std::io::{self, Stdout};

use post_haste::agent::{Agent, Inbox};

use crate::{Addresses, Payloads, postmaster, sequencer};

use hardware::{PedestrianLights, TrafficLights};

#[derive(Debug)]
pub(crate) enum LightsMessage {
    SetTrafficLightState(sequencer::TrafficSequenceState),
    SetPedestrianLightState(sequencer::PedestrianCrossingSequenceState),
    SetButtonLightState(bool),
    Display,
    AddMessage(String),
}

// The TrafficLights and PedestrianLights structs are encapsulated in a module
// to prevent invalid states being created.
// The struct members are accessed through getter functions as the members are
// kept private
mod hardware {
    use crate::sequencer;

    // The state of each light in the traffic lights
    // For an embedded software system, these would be the states of each GPIO
    pub(super) struct TrafficLights {
        red: bool,
        amber: bool,
        green: bool,
    }

    // The traffic lights default into the red state
    impl Default for TrafficLights {
        fn default() -> Self {
            sequencer::TrafficSequenceState::Red.into()
        }
    }

    // TrafficLights can only be created from the 4 valid states, encapsulated in
    // the TrafficSequenceState enum
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
                },
                sequencer::TrafficSequenceState::Green => TrafficLights {
                    red: false,
                    amber: false,
                    green: true,
                },
                sequencer::TrafficSequenceState::GreenToRed => TrafficLights {
                    red: false,
                    amber: true,
                    green: false,
                },
            }
        }
    }

    // 'getter' functions for private struct members
    impl TrafficLights {
        pub(super) fn red(&self) -> bool {
            self.red
        }
        pub(super) fn amber(&self) -> bool {
            self.amber
        }
        pub(super) fn green(&self) -> bool {
            self.green
        }
    }

    // The state of each light in the pedestrian lights
    pub(super) struct PedestrianLights {
        stop: bool,
        cross: bool,
    }

    // The pedestrian lights should default into the stop state
    impl Default for PedestrianLights {
        fn default() -> Self {
            sequencer::PedestrianCrossingSequenceState::Stop.into()
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
                | sequencer::PedestrianCrossingSequenceState::CrossPending => PedestrianLights {
                    stop: true,
                    cross: false,
                },
            }
        }
    }

    // 'getter' functions for the the private struct members
    impl PedestrianLights {
        pub(super) fn stop(&self) -> bool {
            self.stop
        }
        pub(super) fn cross(&self) -> bool {
            self.cross
        }
    }
}

pub(crate) struct LightsAgent {
    traffic_light_state: hardware::TrafficLights,
    pedestrian_light_state: hardware::PedestrianLights,
    cross_pending: bool,
    standard_out: Stdout,
    messages: Vec<String>,
}

impl Agent for LightsAgent {
    type Address = Addresses;
    type Message = postmaster::Message;
    type Config = ();

    async fn create(_address: Self::Address, _config: Self::Config) -> Self {
        Self {
            traffic_light_state: TrafficLights::default(),
            pedestrian_light_state: PedestrianLights::default(),
            cross_pending: false,
            standard_out: io::stdout(),
            messages: vec![String::new()],
        }
    }

    async fn run(mut self, mut inbox: Inbox<Self::Message>) -> ! {
        loop {
            if let Some(message) = inbox.recv().await {
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
                    self.traffic_light_state = TrafficLights::from(traffic_light_sequencer_state)
                }
                LightsMessage::SetPedestrianLightState(pedestrian_crossing_sequencer_state) => {
                    self.pedestrian_light_state =
                        PedestrianLights::from(pedestrian_crossing_sequencer_state)
                }
                LightsMessage::SetButtonLightState(cross_pending) => {
                    self.cross_pending = cross_pending
                }
                LightsMessage::Display => {
                    // Display will update after match statement
                }
                LightsMessage::AddMessage(message) => {
                    while self.messages.len() >= crate::consts::MAX_MESSAGES {
                        self.messages.remove(0);
                    }
                    self.messages.push(message);
                }
            }
            self.display_ascii();
        }
    }

    fn display_ascii(&mut self) {
        // ----
        // |██|   -------
        // ----   |STOP |
        // |██|   |CROSS|
        // ----   -------
        // |██|
        // ----   |██|

        let red_char = if self.traffic_light_state.red() {
            "██".red()
        } else {
            "██".black()
        };
        let amber_char = if self.traffic_light_state.amber() {
            "██".yellow()
        } else {
            "██".black()
        };
        let green_char = if self.traffic_light_state.green() {
            "██".green()
        } else {
            "██".black()
        };

        let stop_chars = if self.pedestrian_light_state.stop() {
            "STOP ".red()
        } else {
            "     ".red()
        };
        let cross_chars = if self.pedestrian_light_state.cross() {
            "CROSS".green()
        } else {
            "     ".green()
        };

        let button_light_chars = if self.cross_pending {
            "██".red()
        } else {
            "  ".red()
        };

        // Clear the terminal (technically this just prints a load of empty lines)
        if let Err(e) = execute!(self.standard_out, terminal::Clear(terminal::ClearType::All)) {
            println!("Error printing to terminal: {:?}", e);
        }

        self.messages
            .iter()
            .for_each(|message| println!("{}", message));

        println!("");
        println!("----");
        println!("|{red_char}|   -------");
        println!("----   |{stop_chars}|");
        println!("|{amber_char}|   |{cross_chars}|");
        println!("----   -------");
        println!("|{green_char}|");
        println!("----   |{button_light_chars}|");
    }
}
