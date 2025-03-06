use esp_backtrace as _;
//use embassy_time::{Duration, Timer};
use esp_hal::peripherals::Peripherals;
use esp_hal::{rmt::Rmt, time::RateExtU32};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::{brightness, gamma, SmartLedsWrite};

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
    let data = [smart_leds::colors::AQUAMARINE; 22];
    led.write(brightness(gamma(data.iter().cloned()), 1))
        .unwrap();

    //   loop {
    //     Timer::after(Duration::from_millis(10000)).await;
    //   }
}
