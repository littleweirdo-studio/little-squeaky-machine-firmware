use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
//use embassy_time::{Duration, Timer};
use esp_hal::peripherals::Peripherals;
use esp_println::println;




#[embassy_executor::task]
pub async fn recorder_task() {
  
    loop {
    }
}
