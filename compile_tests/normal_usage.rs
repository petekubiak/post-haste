use post_haste::init_postmaster;

#[post_haste::payloads]
pub enum AgentPayloads {
    AlmondMessage(almond::AlmondData),
    CashewMessage,
}

#[post_haste::addresses]
#[derive(Copy, Clone)]
pub enum AgentAddresses {
    Almond,
    Cashew,
}

mod almond {
    pub enum AlmondMessage {
        ThingOne(bool),
        ThingTwo(AlmondData),
    }

    pub struct AlmondData {
        f1: u32,
        f2: f64,
    }

    pub struct Almond {
        _address: super::AgentAddresses,
    }

    impl post_haste::agent::Agent for Almond {
        type Address = super::AgentAddresses;
        type Message = super::postmaster::Message;
        type Config = ();

        async fn create(address: Self::Address, _config: Self::Config) -> Self {
            Self { _address: address }
        }

        async fn run(self, mut inbox: post_haste::agent::Inbox<Self::Message>) -> ! {
            loop {
                let received_message = inbox.recv().await.unwrap();
                match received_message.payload {
                    _ => println!("This example doesn't actually do anything!"),
                }
            }
        }
    }
}

post_haste::init_postmaster!();

fn main() {
    // Don't spawn any agents because then the compile test won't end!
}
