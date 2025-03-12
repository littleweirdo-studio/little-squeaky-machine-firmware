use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
//use embassy_time::{Duration, Timer};
use esp_hal::{gpio::{Input, Pin}, peripherals::Peripherals};
use esp_println::println;
use usbd_midi::message::U7;

use crate::MIDI_MESSAGE_CHANNEL;




#[embassy_executor::task]
pub async fn touch_input_task() {
    let mut pin;
    unsafe {
       let peripherals: Peripherals = Peripherals::steal();
       pin = Input::new(peripherals.GPIO0,esp_hal::gpio::Pull::Up);
    }
    loop {
        pin.wait_for_low().await;
        MIDI_MESSAGE_CHANNEL.send(usbd_midi::Message::NoteOn(usbd_midi::message::Channel::Channel1, usbd_midi::message::Note::C2, U7::MAX)).await;
        pin.wait_for_high().await;
        MIDI_MESSAGE_CHANNEL.send(usbd_midi::Message::NoteOff(usbd_midi::message::Channel::Channel1, usbd_midi::message::Note::C2, U7::MAX)).await;
    }
}
