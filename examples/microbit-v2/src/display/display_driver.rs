use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Receiver};
use microbit_bsp::{display::LedMatrix, embassy_nrf::gpio::Output};

use crate::display::DisplayCommand;

#[embassy_executor::task]
pub(super) async fn run(
    mut display: LedMatrix<Output<'static>, 5, 5>,
    inbox: Receiver<'static, NoopRawMutex, DisplayCommand, 1>,
) {
    display.set_brightness(microbit_bsp::display::Brightness::MAX);

    let mut current_command = inbox.receive().await;

    loop {
        current_command = match select(
            process_command(&mut display, current_command),
            inbox.receive(),
        )
        .await
        {
            Either::First(()) => inbox.receive().await,
            Either::Second(next_command) => next_command,
        }
    }
}

async fn process_command(display: &mut LedMatrix<Output<'static>, 5, 5>, command: DisplayCommand) {
    match command {
        DisplayCommand::Text(text) => display.scroll(text).await,
    }
}
