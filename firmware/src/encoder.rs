// use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    peripherals::{Peripherals},
        gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
};
use esp_println::println;

#[embassy_executor::task]
pub async fn encoder_task() {
   let mut enc0_a;
    let mut enc0_b;


    unsafe {
     
        let peripherals: Peripherals = Peripherals::steal();
        let input_pin_config = InputConfig::default().with_pull(Pull::Up);
        enc0_a = Input::new(peripherals.GPIO16, input_pin_config);
        enc0_b = Input::new(peripherals.GPIO8, input_pin_config);
        
    }

    println!("encoder task");
    loop {
            println!("a = {}, b = {}", enc0_a.is_high(), enc0_b.is_high());
          Timer::after(Duration::from_millis(100)).await;
    }
}
