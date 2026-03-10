use alloc::vec;
use driver_cy8cmbr3108::CY8CMBR3108;
use driver_stk8321::STK8321;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    i2c::master::{Config, I2c},
    peripherals::Peripherals,
};
use esp_println::println;
use little_weirdo_gui::{EventType, KeyEvent, MAPPING_SIGNAL};

use crate::{I2c1Bus, keyboard_input::KeyStatus, led::{LED_MESSAGE_CHANNEL, LedMessage}};
extern crate alloc;

#[embassy_executor::task]
pub async fn tilt_task(i2c_bus: &'static I2c1Bus) {
 let i2c_dev = I2cDevice::new(i2c_bus);
let mut stk8321 = STK8321::new(i2c_dev);
    let ready = stk8321.ready().await;
    let _ = stk8321.init().await;
    match ready {
        Ok(true) => println!("STK8321 is ready!"),
        Ok(false) => println!("STK8321 is not ready!"),
        Err(e) => println!("Error checking if STK8321 is ready: {:?}", e),
    }
    loop {
        
          let x = stk8321.read_x().await.unwrap().abs();

            if (x > 100) {
      
                
               // println!("X-axis value: {}", x);
            }

        let y = stk8321.read_y().await.unwrap().abs();
        if (y > 100) {
               // println!("Y-axis value: {}", y);
              /*            LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: smart_leds::colors::ALICE_BLUE,
                        id: 0,
                        brightness: (x / 105) as u8,
                    })
                    .await;*/
                
            }
    
          

        Timer::after(Duration::from_millis(100)).await;
    }
}