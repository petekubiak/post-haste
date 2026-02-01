// The Sequencer Agent, which handles most of the logic for the traffic lights.
// It sends internal messages to itself with a delay, which will trigger the
// next state to be set

use post_haste::agent::Agent;

use crate::display::DisplayMessage;
use crate::{Addresses, Payloads, consts, postmaster};

// An enumeration listing the potential messages that can be sent to the Sequencer agent
#[derive(Debug)]
pub(crate) enum SequencerMessage {
    // Signal for the sequencer to begin
    Begin,
    // Signal to the sequencer that a button press has occured
    ButtonPress,
    // A private message which the Sequencer Agent can send to itself. Other
    // agents cannot send this message
    #[allow(private_interfaces)]
    InternalMessage,
}

// Enumeration defining all the possible states for the system. The system will
// not cycle through all states every cycle
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SequencerState {
    // The traffic light is green, the button has not been pressed. Nothing will
    // happen until the button is pressed. This is the only state which does not
    // have a delayed internal message pending.
    Green,
    // The traffic light is green and the button has been pressed. A delayed internal
    // message will have been sent which will trigger the next state
    GreenCrossPending,
    // The amber light is lit. A delayed internal message is sent
    GreenToRed,
    // The traffic light is red but the pedestrians cannot cross. A delayed message
    // is sent
    RedCrossPending,
    // The traffic light is red and the pedestrians can cross. A delayed message is sent
    RedCrossing,
    // The traffic light is red but the pedestrians cannot cross as the crossing time has
    // ended. A delayed message is sent
    RedCrossEnding,
    // The red and amber lights are lit. A delayed message is sent
    RedToGreen,
    // The red and amber lights are lit, and the button has already been pressed. The
    // next state will be GreenCrossPending. A delayed message is sent
    RedToGreenCrossPending,
}

// The struct for the sequencer agent
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
            // The agent asynchronously awaits for a message to be sent to it in
            // an infinite loop
            let received_message = inbox.recv().await.unwrap();
            match received_message.payload {
                // If a sequencer message is received then it will be handled
                Payloads::Sequencer(message) => self.handle_message(message).await,
                // Any other messages are not valid - this would suggest a mistake in the code
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
            Addresses::DisplayAgent,
            self.address,
            Payloads::Display(DisplayMessage::SetSequenceState {
                sequence_state: self.state.clone(),
            }),
        )
        .await
        .unwrap();
    }

    // The sole purpose of internal messages in this agent is to signal that the
    // sequencer should move onto the next state in the sequence
    async fn handle_internal_message(&mut self) {
        self.set_next_state().await;
        self.send_current_state_to_lights_agent().await;
    }

    async fn handle_button_press(&mut self) {
        match self.state {
            // If the button is pressed in the green state, then change the sequencer
            // state, update the display, and schedule the next internal message
            SequencerState::Green => {
                self.state = SequencerState::GreenCrossPending;
                self.send_current_state_to_lights_agent().await;
                self.schedule_next_state().await;
            }
            // In the following states, nothing should happen when the button is pressed
            SequencerState::GreenCrossPending
            | SequencerState::GreenToRed
            | SequencerState::RedCrossPending
            | SequencerState::RedCrossing
            | SequencerState::RedCrossEnding
            | SequencerState::RedToGreenCrossPending => {
                // Do nothing!
                // Send a debug message - this is just to help understand how this example works
                postmaster::send(
                    Addresses::DisplayAgent,
                    Addresses::SequencerAgent,
                    Payloads::Display(DisplayMessage::DebugMessage(String::from(
                        "Sequencer ignored the button press",
                    ))),
                )
                .await
                .unwrap()
            }
            // Special case if the pedestrian presses the button in the Red to Green state
            SequencerState::RedToGreen => {
                self.state = SequencerState::RedToGreenCrossPending;
                self.send_current_state_to_lights_agent().await;
                // Do not schedule next state - in RedToGreen state, a delayed
                // message will already have been sent!
            }
        }
    }

    // Function to begin the sequencer
    async fn begin(&mut self) {
        self.send_current_state_to_lights_agent().await;
        self.schedule_next_state().await;

        // Also send a debug message - this is just to help understand how this example works
        postmaster::send(
            Addresses::DisplayAgent,
            Addresses::SequencerAgent,
            Payloads::Display(DisplayMessage::DebugMessage(String::from(
                "Sequencer received message to begin sequencing",
            ))),
        )
        .await
        .unwrap()
    }

    // For each state, set the new state and in some instances send a delayed
    // internal message
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
        // In all cases the display agent should be updated
        self.send_current_state_to_lights_agent().await;

        // Also send a debug message - this is just to help understand how this example works
        postmaster::send(
            Addresses::DisplayAgent,
            Addresses::SequencerAgent,
            Payloads::Display(DisplayMessage::DebugMessage(String::from(format!(
                "Sequencer updated it's state to {:?}",
                self.state
            )))),
        )
        .await
        .unwrap()
    }

    // Helper function to send delayed internal messages (from Sequencer Agent
    // to Sequencer Agent). The delay depends on the current state
    async fn schedule_next_state(&mut self) {
        postmaster::message(
            self.address,
            self.address,
            Payloads::Sequencer(SequencerMessage::InternalMessage),
        )
        .with_delay(match self.state {
            SequencerState::Green => unreachable!(),
            SequencerState::GreenCrossPending => consts::GREEN_TO_AMBER_DELAY,
            SequencerState::GreenToRed => consts::AMBER_TO_RED_DELAY,
            SequencerState::RedCrossPending => consts::CROSSING_START_DELAY,
            SequencerState::RedCrossing => consts::CROSSING_LENGTH,
            SequencerState::RedCrossEnding => consts::CROSSING_END_DELAY,
            SequencerState::RedToGreen | SequencerState::RedToGreenCrossPending => {
                consts::AMBER_TO_GREEN_DELAY
            }
        })
        .send()
        .await
        .unwrap();

        // Also send a debug message - this is just to help understand how this example works
        postmaster::send(
            Addresses::DisplayAgent,
            Addresses::SequencerAgent,
            Payloads::Display(DisplayMessage::DebugMessage(String::from(
                "Sequencer agent sent a delayed internal message to itself",
            ))),
        )
        .await
        .unwrap();
    }
}
