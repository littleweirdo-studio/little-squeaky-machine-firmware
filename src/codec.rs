extern crate alloc;
use core::ptr::read;

use driver_sgtl5000::SGTL5000;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::Peripherals;
use esp_println::println;

#[embassy_executor::task]
pub async fn codec_task() {
    let i2c;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let config = Config::default();
        i2c = I2c::new(peripherals.I2C0, config)
            .unwrap()
            .with_sda(peripherals.GPIO47)
            .with_scl(peripherals.GPIO48);
    }

    let mut sgtl500 = SGTL5000::new(i2c);
    let ready = sgtl500.ready();
    match ready {
        Ok(_) => {
            println!(
                "-:= > SGTL5000 :: Chip Revision {:#x}",
                sgtl500.get_chip_revision().unwrap()
            );
            Timer::after(Duration::from_millis(10)).await;
            println!("-:= > SGTL5000 :: Codec setup {:?}", sgtl500.init());
            Timer::after(Duration::from_millis(10)).await;
            println!("-:= > SGTL5000 :: Codec setup {:?}", sgtl500.power_up());
            Timer::after(Duration::from_millis(10)).await;

            let _ = sgtl500.headphone_volume(100);
            // sgtl500.enable_audio_processing();
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
