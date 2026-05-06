//! The purpose of this example is to demonstrate some useful patterns and concepts which take advantage of the post-haste library to great effect.
//! This example uses tokio, however all of the concepts shown here apply equally to Embassy.
//! The logic of this example is very similar to that of the [tokio_basic](tokio_basic.rs) example: simple Agents are created which respond to a "hello" message in kind.
//! "Hello" messages are then sent from the main task to the Agents, with the source address given as one of the other Agents.
//! This will prompt the Agent to respond with its own "hello" back to the source, initiating an infinite loop.

use core::time::Duration;

use polite_agent::{PoliteAgent, PoliteAgentConfig, PoliteAgentMessage};
use post_haste::init_postmaster;
use tokio::time::sleep;

/// This enum describes the messages used by the system.
/// This could be arranged as a single enum of all possible messages, however I prefer to group messages by the actor they are associated with.
/// In this case, as there is only one actor there is only one Payload variant.
#[derive(Debug)]
#[post_haste::payloads]
enum Payloads {
    /// This variant covers all messages associated with the Polite Agent.
    PoliteMessage(PoliteAgentMessage),
    /// This variant is being used to demonstrate the default match case for handling messaged within Agents.
    #[allow(dead_code)]
    Unsupported,
}

/// This enum provides all Agent addresses.
/// Each Agent must be assigned a unique address upon registration with the Postmaster.
/// As indicated by the signature of the Agent trait's `run()` method, Agents are expected to live for the lifetime of the application. This ensures that addresses are always valid and messages aren't accidentally sent to an unoccupied address.
#[post_haste::addresses]
#[derive(Debug, Clone, Copy)]
enum Address {
    AgentA,
    AgentB,
    AgentC,
}

// This macro call generates all the functionality associated with the Postmaster.
// The Address and Payload enum types are provided as arguments, as they are required for the functionality of the Postmaster.
// Note that here the Postmaster is configured with a default timeout of 100 us.
// This third argument is optional and can be omitted, which will result in the Postmaster using a default timeout of 1 ms (1000 us).
// Due to the way the macro expands, if you wish to use a constant to represent the timeout value, you will need to include the scope of the constant, as demonstrated.
const SEND_TIMEOUT_US: u32 = 100;
init_postmaster!(crate::SEND_TIMEOUT_US);

/// This module provides all functionality and types associated with the Polite Agent.
/// Usually this would be in its own file, however here it is presented as a module so that the example is a single, self-reliant file.
mod polite_agent {
    use crate::{Address, Payloads, postmaster};
    use core::time::Duration;
    use post_haste::agent::Agent;

    /// This struct acts as the "instance" of the Agent, and is ususally used to hold state and configuration information
    pub(super) struct PoliteAgent {
        /// Storing the address of the agent as a member field is optional, however it means that `self.address` can be used as the source when sending messages.
        /// In cases with multiple instances of an Agent type are running concurrently, this becomes very necessary.
        address: Address,
        /// Our PoliteAgent can be configured with an optional custom greeting, rather than having to rely on the standard "hello".
        greeting: Option<String>,
        /// In order to avoid spamming stdout with messages, we also configure the PoliteAgent with a delay.
        /// When a message is received, it will wait for this amount of time before sending its reply.
        reply_delay: Duration,
    }

    /// This struct is the associated config for the PoliteAgent.
    /// When implementing the Agent trait, we can provide an associated type called Config, which signals to the Postmaster that we expect the Agent to be configured with this config struct when the Agent is registered and instantiated
    /// This struct is the associated config for the PoliteAgent.
    /// When implementing the Agent trait, we can provide an associated type called Config, which signals to the Postmaster that we expect the Agent to be configured with this config struct when the Agent is registered and instantiated.
    /// Note that the two fields in this struct correspond to the two fields in the PoliteAgent struct which were described as being used to configure it
    pub(super) struct PoliteAgentConfig {
        pub custom_greeting: Option<String>,
        pub reply_delay: Duration,
    }

    /// This enum provides all the messages associated with the PoliteAgent
    #[derive(Debug)]
    pub(super) enum PoliteAgentMessage {
        /// The standard message.
        /// If a PoliteAgent has no custom greeting, it will fall back to using this as its reply.
        Hello,
        /// If a PoliteAgent has been assigned a custom greeting, it will use this message variant, with can hold the custom greeting.
        Greeting(String),
        /// This pattern allows an Agent to have a set of private messages which, while they are "visible" to the rest of the system, the internal structure is private to the PoliteAgent, meaning that the contained data cannot be accessed by other Agents, and messages of this type cannot be generated by other Agents.
        /// Note the declaration of the `InternalMessage` type below, which does not have `pub` visibility.
        /// By default, Rust will warn you that these inaccessible private messages are being included in a public enum with the `private_interfaces` warning, hence the "allow" decorator.
        #[allow(private_interfaces)]
        Internal(InternalMessage),
    }

    /// This enum contains any messages which are private to the PoliteAgent.
    /// This avoids polluting the public `PoliteAgentMessage` namespace with messages which are not relevant outside of this module.
    #[derive(Debug)]
    enum InternalMessage {
        /// The PoliteAgent sends this message to itself with a delay, so when the message is received it indicates that the timer has expired.
        /// This is a simple  way of running an asynchronous timer without needing to select on multiple futures.
        /// Note that as this uses the Postmaster, timing precision may be affected (if there are other messages in the queue they will be processed first).
        TimerExpired { hello_source: Address },
    }

    impl Agent for PoliteAgent {
        /// This associates the Agent with the Address enum defined in your project.
        type Address = Address;
        /// This associates the Agent with the Message type generated by the `init_postmaster!()` call.
        /// The Message type is composed of a source address and a Payload.
        /// This is the same Payload type defined in your project and passed as an argument to `init_postmaster!()`
        type Message = postmaster::Message;
        /// This is an optional type association which allows for passing of any configuration data into the Agent on initialisation.
        /// If this is not required, the unit type `()` can be used.
        /// Please note: the Agent's address is passed in as a separate argument to the Agent's `create()` function, so it does not need to be included in the Agent's Config type.
        type Config = PoliteAgentConfig;

        /// This function instantiates the Agent.
        /// As such, any configuration and initialisation of the Agent should be included in this function.
        /// Usually, you will not need to call this function directly: instead, it is called automatically by the postmaster as a part of `postmaster::register_agent!()`
        /// In the case of the PoliteAgent, we have two configurable attributes of the Agent: a custom greeting message and a configurable delay before it sends its reply.
        /// The Agent's create function extracts these values from the associated Config type and uses them to create an instance of Self.
        async fn create(address: Self::Address, config: Self::Config) -> Self {
            Self {
                address,
                greeting: config.custom_greeting,
                reply_delay: config.reply_delay,
            }
        }

        /// This is the Agent's main loop.
        /// As indicated by the "never" return type this function is expected not to return, meaning that the Agent is expected to exist for the full lifetime of the application.
        /// The function takes ownership of `self`, which makes sure that the instance of the Agent is fully encapsulated and has complete control over its own state.
        /// The other argument this function accepts is the Agent's Inbox (the rx side of the channel).
        /// Again, this function is called automatically by the postmaster as part of the `register_agent!()` call.
        /// A typical `run()` function will be an infinite loop where each iteration of the loop awaits a new message, matches on the message type and then calls a handler to deal with that message.
        /// Some Agents may need to monitor other asynchronous events concurrently with their inbox.
        /// In these cases, the events cannot be awaited consecutively, as for example this may mean that the Agent will not process any incoming messages until it has handled a separate future which it was awaiting.
        /// Instead, use `select` to await all futures simultaneously, handling whichever one resolves first.
        async fn run(mut self, mut inbox: post_haste::agent::Inbox<Self::Message>) -> ! {
            loop {
                let received_message = inbox.recv().await.unwrap();
                match received_message.payload {
                    crate::Payloads::PoliteMessage(PoliteAgentMessage::Hello) => {
                        self.handle_hello(received_message.source).await
                    }
                    crate::Payloads::PoliteMessage(PoliteAgentMessage::Greeting(greeting)) => {
                        self.handle_greeting(received_message.source, &greeting)
                            .await
                    }
                    crate::Payloads::PoliteMessage(PoliteAgentMessage::Internal(
                        InternalMessage::TimerExpired { hello_source },
                    )) => self.send_reply(hello_source).await,
                    // It is advisable to create a generic handler for unsupported messages to make this arm quick and easy to implement.
                    _ => crate::handle_unsupported(self.address, received_message),
                }
            }
        }
    }

    /// This block contains all of the PoliteAgent's message handlers
    impl PoliteAgent {
        /// The Agent has received a generic hello message from somewhere.
        /// All this function needs to do is report that the message was received and then start a timer for the configured delay before replying.
        async fn handle_hello(&mut self, source: Address) {
            println!("{:?} got hello from {source:?}", self.address);
            self.start_timer(source).await
        }

        /// The Agent has received a fancy hello message with a custom greeting!
        /// It reads out the greeting, and then triggers the timer in the same way as the handler above.
        async fn handle_greeting(&mut self, source: Address, greeting: &str) {
            println!(
                "{:?} got greeting from {source:?}: {greeting}",
                self.address
            );
            self.start_timer(source).await
        }

        /// The Agent has been notified that a reply timer has expired and it is now time to send the reply.
        /// The `TimerExpired` message contains information on the original source of the message which needs to be replied to, so this is used as the destination in this function.
        /// First, the function checks whether the Agent is configured with a custom greeting.
        /// If so, this is used to generate a `Greeting` message payload.
        /// Otherwise, a generic `Hello` message payload is used.
        /// Payload defined, the polite reply can then be sent.
        async fn send_reply(&mut self, destination: Address) {
            let payload = if let Some(greeting) = &self.greeting {
                Payloads::PoliteMessage(PoliteAgentMessage::Greeting(greeting.to_string()))
            } else {
                Payloads::PoliteMessage(PoliteAgentMessage::Hello)
            };

            postmaster::send(destination, self.address, payload)
                .await
                .unwrap();
        }
    }

    /// Creating a new block here isn't strictly necessary, however I like to keep message handlers in one block and other methods in a separate block
    impl PoliteAgent {
        /// When a PoliteAgent receives a greeting, it waits for a respectable amount of time before replying.
        /// This delay is configured as part of the Config struct passed in on registration of the Agent.
        /// Simply calling `tokio::time::sleep().await` will result in poor performance of the Agent, as it won't be able to handle any new messages arriving in its inbox until it is done waiting to send its reply to the previous one.
        /// Another option would be to acquire the future from `tokio::time::sleep()` and then use `select` on it along with the inbox future in order to monitor both futures concurrently.
        /// This gets complicated very quickly however.
        /// A much simpler way is to use the Postmaster's message builder to build a message with a delay.
        /// The Agent builds a `TimerExpired` message, setting itself as the destination, and configures the message to be delayed by its configured amount.
        /// This means the Agent can immediately finish handling the message it just received and return to monitoring its inbox, safe in the knowledge that once the delay period is over, the TimerExpired message will appear in its inbox, prompting it to send its reply.
        async fn start_timer(&mut self, hello_source: Address) {
            postmaster::message(
                self.address,
                self.address,
                Payloads::PoliteMessage(PoliteAgentMessage::Internal(
                    InternalMessage::TimerExpired { hello_source },
                )),
            )
            .with_delay(self.reply_delay)
            .send()
            .await
            .unwrap();
        }
    }
}

/// In the `main` function, we configure and register each of our Agents.
/// Then, in order to get them to start talking to each other, we send a hello to each with the source set to one of the others.
/// This will start a never-ending cycle where the Agents will exchange pleasant greetings!
#[tokio::main]
async fn main() {
    // This agent is configured with no custom greeting, so will reply with a generic hello message
    postmaster::register_agent!(
        AgentA,
        PoliteAgent,
        PoliteAgentConfig {
            custom_greeting: None,
            reply_delay: Duration::from_secs(1)
        }
    )
    .unwrap();
    // This agent has a fancy custom greeting, so will use this in its replies instead!
    postmaster::register_agent!(
        AgentB,
        PoliteAgent,
        PoliteAgentConfig {
            custom_greeting: Some("Good day!".to_string()),
            reply_delay: Duration::from_secs(2)
        }
    )
    .unwrap();
    // As well as having its own custom greeting, this Agent also has been given an expanded message queue size!
    // Each Agent message queue can be sized independently.
    // The message queues are held in static memory, so the larger the mailbox the more static memory will be needed.
    // This is not so much of a concern for applications running on a full OS, but is an important consideration in resource-constrained embedded devices.
    postmaster::register_agent!(
        AgentC,
        PoliteAgent,
        PoliteAgentConfig {
            custom_greeting: Some("Ahoy!".to_string()),
            reply_delay: Duration::from_secs(3)
        },
        2
    )
    .unwrap();

    postmaster::send(
        Address::AgentA,
        Address::AgentB,
        Payloads::PoliteMessage(PoliteAgentMessage::Hello),
    )
    .await
    .unwrap();
    postmaster::send(
        Address::AgentB,
        Address::AgentC,
        Payloads::PoliteMessage(PoliteAgentMessage::Hello),
    )
    .await
    .unwrap();
    postmaster::send(
        Address::AgentC,
        Address::AgentA,
        Payloads::PoliteMessage(PoliteAgentMessage::Hello),
    )
    .await
    .unwrap();

    // This loop makes sure that the application continues to run indefinitely, until a termination signal is received.
    // It showcases the `diagnostics()` function, which simply returns a running total of the number of messages successfully sent, as well as a count of any send failures.
    // The idea of this is as a high-level overview for the developer, to keep an eye on whether there are any communication issues within the application.
    // Failures are most commonly caused by timeouts due to full mailboxes - the error Result type returned by the Postmaster in these instances gives more information on the cause.
    loop {
        sleep(Duration::from_secs(60)).await;
        let diagnostics = postmaster::get_diagnostics();
        println!("Postmaster diagnostics:");
        println!("Messages sent: {}", diagnostics.messages_sent);
        println!("Send failures: {}", diagnostics.send_failures);
    }
}

/// Simple helper function to alert the developer that an unexpected message was received by an Agent.
fn handle_unsupported(my_address: Address, message: postmaster::Message) {
    eprintln!(
        "Agent {my_address:?} got an unsupported message from {:?}: {:?}",
        message.source, message.payload
    )
}
