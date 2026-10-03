use driver_cy8cmbr3108::CY8CMBR3108;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::delay::Delay;
use esp_println::println;
use little_weirdo_gui::{Event, MAPPING_SIGNAL};
use crate::{I2c1Bus, keyboard_input::{KEYPAD_EVENT_DOWN, KEYPAD_EVENT_UP, KeyStatus}};
extern crate alloc;

#[embassy_executor::task]
pub async fn touch_input_task(i2c_bus: &'static I2c1Bus) {
    let i2c_dev = I2cDevice::new(i2c_bus);

    let delay = Delay::new();

    let mut touch = CY8CMBR3108::new(i2c_dev, delay);
    let _ = touch.wake_up();

    //device id = 2563
    println!(
        "CY8CMBR3108 Device Identifier : {:?} (=2563)",
        touch.get_device_id().await.unwrap()
    );
    // revision = 0001
    println!(
        "CY8CMBR3108 Device Revision   : {:04} (=0001)",
        touch.get_device_revision().await.unwrap()
    );

    let is_config_ready = touch.write_configuration().await;
    match is_config_ready {
        Ok(true) => {
            println!("CY8CMBR3108 Configuration successful!");
        }
        Ok(false) => {
            println!("CY8CMBR3108 Configuration failed.")
        }
        Err(e) => println!("CY8CMBR3108 Configuration failed with error: {:?}", e),
    }

    let mut key_status_array: [KeyStatus; 8] = [KeyStatus::default(); 8];

    loop {
        Timer::after(Duration::from_millis(10)).await;

        let status = touch.get_button_status().await.unwrap();

        // Check each bit individually
        for i in 0..8 {
            if (status & (1 << i)) != 0 {
                if !key_status_array[i].key_down_trigger {
                    println!("touch down {:}", i);
                    key_status_array[i].key_down_trigger = true;
                     let event = Event {
                                event_identifier: KEYPAD_EVENT_DOWN,
                                id: i as u8,
                                value: 0,
                            };

                   
                    let _ = MAPPING_SIGNAL.try_send(event);
                }
            } else if key_status_array[i].key_down_trigger {
                key_status_array[i].key_down_trigger = false;
                                    println!("touch up {:}", i);

                let event = Event {
                                event_identifier: KEYPAD_EVENT_UP,
                                id: i as u8,
                                value: 0,
                            };

                   
                    let _ = MAPPING_SIGNAL.try_send(event);
            }
        }
    }
}
