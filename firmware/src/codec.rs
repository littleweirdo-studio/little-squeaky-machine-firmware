extern crate alloc;

use driver_sgtl5000::SGTL5000;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::Peripherals;
use esp_println::println;

use crate::I2c1Bus;

#[embassy_executor::task]
pub async fn codec_task(i2c_bus: &'static I2c1Bus ) {
    let i2c_dev = I2cDevice::new(i2c_bus);

    let mut sgtl500 = SGTL5000::new(i2c_dev);
    let ready = sgtl500.ready().await;
    match ready {
        Ok(_) => {
            println!(
                "-:= > SGTL5000 :: Chip Revision {:#x}",
                sgtl500.get_chip_revision().await.unwrap()
            );
            println!("-:= > SGTL5000 :: Codec init {:?}", sgtl500.init().await);
            println!("-:= > SGTL5000 :: Codec power_up {:?}", sgtl500.power_up().await);
            let _ = sgtl500.headphone_volume(100).await;
            //sgtl500.enable_audio_processing().await;
            sgtl500.enable_bass_enhance().await;
        
        }
        Err(_) => {
            println!("-:= > SGTL5000 :: Not ready");
            println!("-:= > SGTL5000 :: Make sure you provide the NXP SGTL5000 with a MCLK signal first before using I2C.")
        }
    }

    loop {
        Timer::after(Duration::from_millis(50)).await;
    }
}
