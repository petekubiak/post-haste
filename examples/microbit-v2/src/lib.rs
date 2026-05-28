#![no_std]

use embassy_executor::Spawner;
use microbit_bsp::Microbit;
use post_haste::init_postmaster;

use crate::display::{DisplayAgent, DisplayAgentConfig, DisplayCommand};

mod display;

#[derive(defmt::Format)]
#[post_haste::addresses]
pub enum Addresses {
    None,
    Display,
}

#[derive(defmt::Format)]
#[post_haste::payloads]
enum Payloads {
    Display(DisplayCommand),
}

post_haste::init_postmaster!();

pub async fn run(spawner: Spawner) {
    let board = Microbit::default();

    let display = board.display;

    defmt::unwrap!(
        postmaster::register_agent!(
            spawner,
            Display,
            DisplayAgent,
            DisplayAgentConfig { display, spawner },
            1
        ),
        "Unable to register display agent"
    );

    // let mut btn_a = board.btn_a;
    // let mut btn_b = board.btn_b;

    postmaster::send(
        Addresses::Display,
        Addresses::None,
        Payloads::Display(DisplayCommand::Text("Hello, world!")),
    )
    .await
    .unwrap();

    // defmt::info!("Application started, press buttons!");
    // loop {
    //     match select(btn_a.wait_for_low(), btn_b.wait_for_low()).await {
    //         Either::First(_) => {
    //             defmt::info!("A pressed");
    //             display
    //                 .display(display::fonts::ARROW_LEFT, Duration::from_secs(1))
    //                 .await;
    //         }
    //         Either::Second(_) => {
    //             defmt::info!("B pressed");
    //             display
    //                 .display(display::fonts::ARROW_RIGHT, Duration::from_secs(1))
    //                 .await;
    //         }
    //     }
}
