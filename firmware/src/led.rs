use embassy_futures::select::{Either, select};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use esp_backtrace as _;
use esp_hal::peripherals::Peripherals;
use esp_hal::{rmt::Rmt, time::Rate};
use esp_hal_smartled::{SmartLedsAdapter, smart_led_buffer};
use smart_leds::{RGB8, SmartLedsWrite, brightness, colors, gamma};

// channel for sharing led update message between tasks and processes running on different cores.
#[derive(Debug)]
pub struct LedMessage {
    pub color: RGB8,
    pub id: u8,
}

pub static LED_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, LedMessage, 10> = Channel::new();
pub static LED_BRIGHTNESS_CHANNEL: Channel<CriticalSectionRawMutex, u8, 1> = Channel::new();

// total leds in the led chains
pub const AMOUNT_OF_LEDS: usize = 35;
// initial brightness of the leds
pub const INITIAL_BRIGHTNESS: u8 = 10;
// the two leds left of encoders
pub const LED_ID_GENERAL_0: usize = 0;
pub const LED_ID_GENERAL_1: usize = 1;
// the leds below the four encoders
pub const LED_ID_ENCODER_0: usize = 27;
pub const LED_ID_ENCODER_1: usize = 28;
pub const LED_ID_ENCODER_2: usize = 29;
pub const LED_ID_ENCODER_3: usize = 30;

#[embassy_executor::task]
pub async fn led_task() {
    /*
    | Pin | 21 | RMT | 💡 Led controller |
     */
    let rmt;
    let led_pin;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let freq = Rate::from_mhz(80);
        rmt = Rmt::new(peripherals.RMT, freq).unwrap();
        led_pin = peripherals.GPIO21;
    }

    let mut rmt_buffer = smart_led_buffer!(AMOUNT_OF_LEDS);
    let mut led = SmartLedsAdapter::new(rmt.channel0, led_pin, &mut rmt_buffer);
    let mut data: [RGB8; AMOUNT_OF_LEDS] = [colors::BLACK; AMOUNT_OF_LEDS];
    data[LED_ID_GENERAL_0] = colors::GREEN;
    data[LED_ID_GENERAL_1] = colors::YELLOW;

    // the four encoders have a led underneath, from left to right (enc0, enn1, enc2, enc3)
    //enc3
    data[LED_ID_ENCODER_0] = colors::RED;
    //enc2
    data[LED_ID_ENCODER_1] = colors::PINK;
    //enc1
    data[LED_ID_ENCODER_2] = colors::BLUE;
    //enc0
    data[LED_ID_ENCODER_3] = colors::YELLOW;

    let mut brightness_level = INITIAL_BRIGHTNESS;
    led.write(brightness(gamma(data.iter().cloned()), brightness_level))
        .unwrap();

    loop {
        match select(
            LED_MESSAGE_CHANNEL.receive(),
            LED_BRIGHTNESS_CHANNEL.receive(),
        )
        .await
        {
            Either::First(message) => data[message.id as usize] = message.color,
            Either::Second(new_brightness) => brightness_level = new_brightness,
        }

        led.write(brightness(gamma(data.iter().cloned()), brightness_level))
            .unwrap();
    }
}
