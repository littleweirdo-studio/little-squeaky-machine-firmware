#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

//
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::Peripherals;
use esp_hal::{delay::Delay, main, rmt::Rmt, time::RateExtU32};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use esp_println::println;
use smart_leds::{brightness, gamma, SmartLedsWrite};

//
use esp_backtrace as _;
//

use esp_hal::timer::{timg::TimerGroup, AnyTimer};
//

#[embassy_executor::task]
pub async fn led_task() {
    let rmt;
    let led_pin;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let freq = 80.MHz();
        rmt = Rmt::new(peripherals.RMT, freq).unwrap();
        led_pin = peripherals.GPIO21;

        // SmartLed crate including
    }

    // We use one of the RMT channels to instantiate a `SmartLedsAdapter` which can
    // be used directly with all `smart_led` implementations
    let rmt_buffer = smartLedBuffer!(22);

    let mut led = SmartLedsAdapter::new(rmt.channel0, led_pin, rmt_buffer);

    let data = [smart_leds::colors::RED; 22];

    println!("test led loop");
    loop {
        led.write(brightness(gamma(data.iter().cloned()), 5))
            .unwrap();
        Timer::after(Duration::from_millis(1000)).await;
        led.write(brightness(gamma(data.iter().cloned()), 0))
            .unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    println!("test led");
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::_240MHz);
    let peripherals = esp_hal::init(config);

    // TODO: do we need 4 timers?
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer_group_1 = TimerGroup::new(peripherals.TIMG1);

    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    let timer3: AnyTimer = timer_group_1.timer0.into();
    let timer4: AnyTimer = timer_group_1.timer1.into();

    esp_hal_embassy::init([timer1, timer2, timer3, timer4]);

    // spawn the tasks
    Timer::after(Duration::from_millis(500)).await;
    let _ = spawner.spawn(led_task());
}
