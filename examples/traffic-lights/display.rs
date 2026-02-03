//! The Display Agent is responsible for displaying text in the terminal

// Help with clearing the terminal
use crossterm::{execute, style::Stylize, terminal};
use std::io::{self, Stdout};
// Library for getting the current time - this is for the debug messages
use chrono::Local;

use crate::{Addresses, Payloads, postmaster, sequencer};
use hardware::{ButtonLight, PedestrianLights, TrafficLights};
use post_haste::agent::{Agent, Inbox};

/// Declares valid messages which can be sent to this agent
#[derive(Debug)]
pub(crate) enum DisplayMessage {
    /// Update the display with the current sequencer state
    SetSequenceState {
        sequence_state: sequencer::SequencerState,
    },
    /// Add a message which will be displayed above the ascii traffic lights
    DebugMessage(String),
}

/// The TrafficLights and PedestrianLights structs are encapsulated in a module
/// to prevent invalid states being created. For example, the red and green lights
/// cannot be active both at the same time!
/// The struct members are accessed through getter functions as the members are
/// kept private, to avoid anyone modifying this code in the future from accidentally
/// allowing invalid states
mod hardware {
    use crate::sequencer;

    #[derive(Copy, Clone, Eq, PartialEq)]
    pub enum LightState {
        Off,
        On,
    }

    /// The state of each light in the traffic lights
    /// For an embedded software system, these would be the states of each GPIO
    pub(super) struct TrafficLights {
        red: LightState,
        amber: LightState,
        green: LightState,
    }

    // The traffic lights default into the red state
    impl Default for TrafficLights {
        fn default() -> Self {
            sequencer::SequencerState::RedCrossEnding.into()
        }
    }

    // Implement convertion from the sequencer state enum into the display struct
    // of LightStates
    impl From<sequencer::SequencerState> for TrafficLights {
        fn from(value: sequencer::SequencerState) -> Self {
            match value {
                sequencer::SequencerState::Green | sequencer::SequencerState::GreenCrossPending => {
                    Self {
                        red: LightState::Off,
                        amber: LightState::Off,
                        green: LightState::On,
                    }
                }
                sequencer::SequencerState::GreenToRed => Self {
                    red: LightState::Off,
                    amber: LightState::On,
                    green: LightState::Off,
                },
                sequencer::SequencerState::RedCrossPending
                | sequencer::SequencerState::RedCrossing
                | sequencer::SequencerState::RedCrossEnding => Self {
                    red: LightState::On,
                    amber: LightState::Off,
                    green: LightState::Off,
                },
                sequencer::SequencerState::RedToGreen
                | sequencer::SequencerState::RedToGreenCrossPending => Self {
                    red: LightState::On,
                    amber: LightState::On,
                    green: LightState::Off,
                },
            }
        }
    }

    // 'getter' functions for private struct members
    impl TrafficLights {
        pub(super) fn red(&self) -> LightState {
            self.red
        }
        pub(super) fn amber(&self) -> LightState {
            self.amber
        }
        pub(super) fn green(&self) -> LightState {
            self.green
        }
    }

    /// The state of each light in the pedestrian lights
    pub(super) struct PedestrianLights {
        stop: LightState,
        cross: LightState,
    }

    /// The pedestrian lights should default into the stop state
    impl Default for PedestrianLights {
        fn default() -> Self {
            sequencer::SequencerState::RedCrossEnding.into()
        }
    }

    // Implement conversion from the sequencer state enum into the display struct
    // of LightStates
    impl From<sequencer::SequencerState> for PedestrianLights {
        fn from(value: sequencer::SequencerState) -> Self {
            match value {
                sequencer::SequencerState::Green
                | sequencer::SequencerState::GreenCrossPending
                | sequencer::SequencerState::GreenToRed
                | sequencer::SequencerState::RedCrossPending
                | sequencer::SequencerState::RedCrossEnding
                | sequencer::SequencerState::RedToGreen
                | sequencer::SequencerState::RedToGreenCrossPending => Self {
                    stop: LightState::On,
                    cross: LightState::Off,
                },
                sequencer::SequencerState::RedCrossing => Self {
                    stop: LightState::Off,
                    cross: LightState::On,
                },
            }
        }
    }

    // 'getter' functions for the the private struct members
    impl PedestrianLights {
        pub(super) fn stop(&self) -> LightState {
            self.stop
        }
        pub(super) fn cross(&self) -> LightState {
            self.cross
        }
    }

    #[derive(PartialEq, Eq)]
    pub struct ButtonLight {
        pub(super) button_light: LightState,
    }

    impl Default for ButtonLight {
        fn default() -> Self {
            sequencer::SequencerState::RedCrossEnding.into()
        }
    }

    impl From<sequencer::SequencerState> for ButtonLight {
        fn from(value: sequencer::SequencerState) -> Self {
            match value {
                sequencer::SequencerState::RedToGreenCrossPending
                | sequencer::SequencerState::GreenCrossPending
                | sequencer::SequencerState::GreenToRed
                | sequencer::SequencerState::RedCrossPending => Self {
                    button_light: LightState::On,
                },
                sequencer::SequencerState::Green
                | sequencer::SequencerState::RedCrossing
                | sequencer::SequencerState::RedCrossEnding
                | sequencer::SequencerState::RedToGreen => Self {
                    button_light: LightState::Off,
                },
            }
        }
    }
}

/// The Display Agent
pub(crate) struct DisplayAgent {
    traffic_light_state: hardware::TrafficLights,
    pedestrian_light_state: hardware::PedestrianLights,
    button_state: hardware::ButtonLight,
    standard_out: Stdout,
    debug_messages: Vec<String>,
}

impl Agent for DisplayAgent {
    type Address = Addresses;
    type Message = postmaster::Message;
    type Config = ();

    async fn create(_address: Self::Address, _config: Self::Config) -> Self {
        Self {
            traffic_light_state: TrafficLights::default(),
            pedestrian_light_state: PedestrianLights::default(),
            button_state: ButtonLight::default(),
            standard_out: io::stdout(),
            debug_messages: vec![String::new()],
        }
    }

    async fn run(mut self, mut inbox: Inbox<Self::Message>) -> ! {
        loop {
            if let Some(message) = inbox.recv().await {
                // Await messages
                match message.payload {
                    Payloads::Display(lights_message) => self.message_handler(lights_message),
                    _ => println!(
                        "DisplayAgent received unsupported message {:?}",
                        message.payload
                    ),
                }
            }
        }
    }
}

impl DisplayAgent {
    /// Handle each type of message that can be sent to this agent
    fn message_handler(&mut self, lights_message: DisplayMessage) {
        match lights_message {
            DisplayMessage::SetSequenceState { sequence_state } => {
                self.traffic_light_state = sequence_state.clone().into();
                self.pedestrian_light_state = sequence_state.clone().into();
                self.button_state = sequence_state.clone().into();
            }
            DisplayMessage::DebugMessage(debug_message) => {
                while self.debug_messages.len() >= crate::consts::MAXIMUM_DEBUG_MESSAGES {
                    self.debug_messages.remove(0);
                }
                // Prepend the message with the current time
                self.debug_messages.push(format!(
                    "{} {}",
                    Local::now().format("%H:%M:%S"),
                    debug_message
                ));
            }
        }
        self.display_ascii();
    }

    /// Function to display debug messages followed by the state of the traffic lights
    /// using ascii
    fn display_ascii(&mut self) {
        use hardware::LightState;
        // ----
        // |██|   -------
        // ----   |STOP |
        // |██|   |CROSS|
        // ----   -------
        // |██|
        // ----   |██|

        let red_char = if self.traffic_light_state.red() == LightState::On {
            "██".red()
        } else {
            "██".black()
        };
        let amber_char = if self.traffic_light_state.amber() == LightState::On {
            "██".yellow()
        } else {
            "██".black()
        };
        let green_char = if self.traffic_light_state.green() == LightState::On {
            "██".green()
        } else {
            "██".black()
        };

        let stop_chars = if self.pedestrian_light_state.stop() == LightState::On {
            "STOP ".red()
        } else {
            "     ".red()
        };
        let cross_chars = if self.pedestrian_light_state.cross() == LightState::On {
            "CROSS".green()
        } else {
            "     ".green()
        };

        let button_light_chars = if self.button_state.button_light == LightState::On {
            "██".red()
        } else {
            "  ".red()
        };

        // Clear the terminal (technically this just prints a load of empty lines)
        if let Err(e) = execute!(self.standard_out, terminal::Clear(terminal::ClearType::All)) {
            println!("Error printing to terminal: {:?}", e);
        }

        self.debug_messages
            .iter()
            .for_each(|message| println!("{}", message));

        println!();
        println!("----");
        println!("|{red_char}|   -------");
        println!("----   |{stop_chars}|");
        println!("|{amber_char}|   |{cross_chars}|");
        println!("----   -------");
        println!("|{green_char}|");
        println!("----   |{button_light_chars}|");
    }
}
