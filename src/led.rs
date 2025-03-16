use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use esp_backtrace as _;
//use embassy_time::{Duration, Timer};
use esp_hal::peripherals::Peripherals;
use esp_hal::{rmt::Rmt, time::RateExtU32};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{brightness, colors, gamma, SmartLedsWrite, RGB8};

// channel for sharing led update message between tasks and processes running on different cores.
#[derive(Debug)]
pub struct LedMessage {
    pub enable: bool,
    pub color: RGB8,
    pub id: u8,
}

pub static LED_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, LedMessage, 10> = Channel::new();

#[embassy_executor::task]
pub async fn led_task() {
    /*
    | Pin | 21 | RMT | 💡 Led controller |
     */
    let rmt;
    let led_pin;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let freq = 80.MHz();
        rmt = Rmt::new(peripherals.RMT, freq).unwrap();
        led_pin = peripherals.GPIO21;
    }

    let rmt_buffer = smartLedBuffer!(22);
    let mut led = SmartLedsAdapter::new(rmt.channel0, led_pin, rmt_buffer);
    let mut data: [RGB8; 22] = [colors::AQUAMARINE; 22];

    led.write(brightness(gamma(data.iter().cloned()), 2))
        .unwrap();

    loop {
        let message = LED_MESSAGE_CHANNEL.receive().await;
        data[message.id as usize] = message.color;

        led.write(brightness(gamma(data.iter().cloned()), 10))
            .unwrap();
    }
}
