use embassy_time::Timer;
use post_haste::agent::{Agent, Inbox};
use rtt_target::rprintln;

use crate::{Address, Payloads, postmaster};

pub(crate) struct PoliteAgent {
    address: Address,
}

impl Agent for PoliteAgent {
    type Address = Address;
    type Message = postmaster::Message;
    type Config = ();

    async fn create(address: Self::Address, _config: Self::Config) -> Self {
        Self { address }
    }

    async fn run(self, inbox: Inbox<Self::Message>) -> ! {
        loop {
            let received_message = inbox.receive().await;
            match &received_message.payload {
                Payloads::Hello => self.handle_hello(received_message.source).await,
            };
        }
    }
}

impl PoliteAgent {
    async fn handle_hello(&self, source: Address) {
        rprintln!("{:?} got hello from {:?}!", self.address, source);
        Timer::after_secs(1).await;
        postmaster::send(source, self.address, Payloads::Hello)
            .await
            .unwrap();
    }
}
