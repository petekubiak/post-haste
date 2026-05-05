#![no_std]
#![no_main]

use panic_probe as _;

use embassy_executor::Spawner;
use rtt_target::rprintln;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    rtt_target::rtt_init_print!();
    let _peripherals = embassy_nrf::init(Default::default());

    loop {
        rprintln!("Hello, world!");
        embassy_time::Timer::after_secs(1).await;
    }
}
