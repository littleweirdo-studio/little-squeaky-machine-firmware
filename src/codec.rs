extern crate alloc;
use driver_sgtl5000::SGTL5000;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::Peripherals;
use esp_println::println;
use fugit::RateExtU32;

#[embassy_executor::task]
pub async fn codec_task() {
    let i2c;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let mut config = Config::default();
        config.frequency = 400u32.kHz();
        i2c = I2c::new(peripherals.I2C0, config)
            .unwrap()
            .with_sda(peripherals.GPIO47)
            .with_scl(peripherals.GPIO48);
    }

    let mut sgtl500 = SGTL5000::new(i2c);
    println!("--> SGTL5000 :: Chip Ready {:?}", sgtl500.ready().unwrap());
    println!("--> SGTL5000 :: Chip Revision {:#x}", sgtl500.get_chip_revision().unwrap());
    println!("--> SGTL5000 :: Codec setup {:?}", sgtl500.init());
 
    sgtl500.headphone_volume(70);
    sgtl500.enable_audio_processing();

    loop {
        Timer::after(Duration::from_millis(50)).await;
    }
}
