//! This example provides a very simple scenario of two Agents exchanging messages with each other.

use core::time::Duration;

use post_haste::init_postmaster;
use tokio::time::sleep;

use crate::polite_agent::PoliteAgent;

#[post_haste::payloads]
enum Payloads {
    Hello,
}

#[derive(Debug, Clone, Copy)]
#[post_haste::addresses]
enum Addresses {
    A,
    B,
}

init_postmaster!(Addresses, Payloads);

#[tokio::main]
async fn main() {
    println!("hello");
    run().await;

    loop {
        println!("Hello world!");
        sleep(Duration::from_secs(1)).await;
    }
}
pub async fn run() {
    postmaster::register_agent!(A, PoliteAgent, ()).unwrap();
    postmaster::register_agent!(B, PoliteAgent, (), 2).unwrap();
    postmaster::send(Addresses::A, Addresses::B, Payloads::Hello)
        .await
        .unwrap();
}

mod polite_agent {
    use post_haste::agent::{Agent, Inbox};
    use tokio::time::Duration;
    use tokio::time::sleep;

    use crate::{Addresses, Payloads, postmaster};

    pub(crate) struct PoliteAgent {
        address: Addresses,
    }

    impl Agent for PoliteAgent {
        type Address = Addresses;
        type Message = postmaster::Message;
        type Config = ();

        async fn create(address: Self::Address, _config: Self::Config) -> Self {
            Self { address }
        }

        async fn run(self, mut inbox: Inbox<Self::Message>) -> ! {
            loop {
                let received_message = inbox.recv().await.unwrap();
                match &received_message.payload {
                    Payloads::Hello => self.handle_hello(received_message.source).await,
                };
            }
        }
    }

    impl PoliteAgent {
        async fn handle_hello(&self, source: Addresses) {
            println!("{:?} got hello from {:?}!", self.address, source);
            sleep(Duration::from_secs(1)).await;
            postmaster::send(source, self.address, Payloads::Hello)
                .await
                .unwrap();
        }
    }
}
