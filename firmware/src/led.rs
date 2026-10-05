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

pub const INITIAL_BRIGHTNESS: u8 = 10;

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

    let mut rmt_buffer = smart_led_buffer!(35);
    let mut led = SmartLedsAdapter::new(rmt.channel0, led_pin, &mut rmt_buffer);
    let mut data: [RGB8; 35] = [colors::BLACK; 35];
    data[0] = colors::GREEN;
    data[1] = colors::GREEN;

    /*data[1] = colors::DEEP_PINK;



    data[27] = colors::DEEP_PINK;
    data[28] = colors::LIGHT_SKY_BLUE;
    data[29] = colors::DEEP_PINK;
    data[30] = colors::LIGHT_SKY_BLUE;

    //
    data[26] = colors::RED;*/

    //reset
    data[26] = colors::BLUE;

    //en0
    data[27] = colors::DEEP_PINK;
    //en1
    data[28] = colors::DEEP_PINK;
    //en2
    data[29] = colors::DEEP_PINK;
    //en3
    data[30] = colors::DEEP_PINK;

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
