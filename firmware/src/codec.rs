extern crate alloc;

use driver_sgtl5000::SGTL5000;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_println::println;

use crate::I2c1Bus;

#[derive(Clone, Debug, PartialEq)]
pub enum CodecEventType {
    ChangeVolume
}


#[derive(Clone, Debug, PartialEq)]
pub struct CodecEvent {
    pub event: CodecEventType,
    pub value: u16
}


pub static CODEC_SIGNAL: Channel<CriticalSectionRawMutex, CodecEvent, 50> = Channel::new();


#[embassy_executor::task]
pub async fn codec_task(i2c_bus: &'static I2c1Bus) {
    let i2c_dev = I2cDevice::new(i2c_bus);
    let volume = 50;
    let mut sgtl500 = SGTL5000::new(i2c_dev);
    for i in 0..10 {
        println!("Starting -:= > SGTL5000  ... {}x try ", i);
        let ready = sgtl500.ready().await;
        match ready {
            Ok(_) => {
                println!(
                    "-:= > SGTL5000 :: Chip Revision {:?}",
                    sgtl500.get_chip_revision().await
                );
                println!("-:= > SGTL5000 :: Codec init {:?}", sgtl500.init().await);
                println!(
                    "-:= > SGTL5000 :: Codec power_up {:?}",
                    sgtl500.power_up().await
                );
                let _ = sgtl500.headphone_volume(volume).await;
               // sgtl500.select_adc_input(driver_sgtl5000::AdcInputSources::LineIn).await;
                let _ = sgtl500.enable_audio_processing().await;
               // sgtl500.enable_mixer().await;
               // sgtl500.set_mix_channel_volume_level().await;
                let _ = sgtl500.enable_surround().await;

                //  sgtl500.enable_bass_enhance().await;
                break;
            }
            Err(_) => {
                Timer::after(Duration::from_millis(1000)).await;
            }
        }
    }

    loop {
        let event = CODEC_SIGNAL.receive().await;
        match event.event {
            CodecEventType::ChangeVolume => {
                 let _ = sgtl500.headphone_volume((event.value / 2) as u8).await;
            }
        }

    }
}
