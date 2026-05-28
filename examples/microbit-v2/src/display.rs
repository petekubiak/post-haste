use defmt::warn;
use embassy_executor::Spawner;
use embassy_sync::channel::Sender;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};
use microbit_bsp::{display::LedMatrix, embassy_nrf::gpio::Output};
use static_cell::StaticCell;

use crate::{Addresses, Payloads, postmaster::Message};

mod display_driver;

#[derive(defmt::Format)]
pub enum DisplayCommand {
    Text(&'static str),
}

pub struct DisplayAgent {
    display: Sender<'static, NoopRawMutex, DisplayCommand, 1>,
}

pub struct DisplayAgentConfig {
    pub display: LedMatrix<Output<'static>, 5, 5>,
    pub spawner: Spawner,
}

impl post_haste::agent::Agent for DisplayAgent {
    type Address = Addresses;
    type Message = Message;
    type Config = DisplayAgentConfig;

    async fn create(_address: Self::Address, config: Self::Config) -> Self {
        let (tx, rx) = {
            static CELL: StaticCell<Channel<NoopRawMutex, DisplayCommand, 1>> = StaticCell::new();
            let channel = CELL.init(Channel::new());
            (channel.sender(), channel.receiver())
        };

        config.spawner.spawn(defmt::expect!(
            display_driver::run(config.display, rx),
            "Failed to spawn display driver"
        ));

        Self { display: tx }
    }

    async fn run(self, inbox: post_haste::agent::Inbox<Self::Message>) -> ! {
        loop {
            let message = inbox.receive().await;
            match message.payload {
                Payloads::Display(command) => self.display.send(command).await,
                _ => warn!(
                    "Display agent received unsupported message from {:?}: {:?}",
                    message.source, message.payload
                ),
            }
        }
    }
}
