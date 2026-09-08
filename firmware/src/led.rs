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
    pub id: u8
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
        let freq = Rate::from_mhz(80);
        rmt = Rmt::new(peripherals.RMT, freq).unwrap();
        led_pin = peripherals.GPIO21;
    }

    let mut rmt_buffer = smart_led_buffer!(31);
    let mut led = SmartLedsAdapter::new(rmt.channel0, led_pin, &mut rmt_buffer);
    let mut data: [RGB8; 31] = [colors::BLACK; 31];
    data[0] = colors::LIGHT_SKY_BLUE;
    data[1] = colors::DEEP_PINK;

    data[27] = colors::DEEP_PINK;
    data[28] = colors::LIGHT_SKY_BLUE;
    data[29] = colors::DEEP_PINK;
    data[30] = colors::LIGHT_SKY_BLUE;
    
    

    //
    data[26] = colors::RED;

    led.write(brightness(gamma(data.iter().cloned()), 30))
        .unwrap();

   
    loop {
        let message = LED_MESSAGE_CHANNEL.receive().await;
        data[message.id as usize] = message.color;

        led.write(brightness(gamma(data.iter().cloned()), 50))
            .unwrap();
    }
}
