#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel as Chn, signal::Signal,
};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    analog::dac::Dac,
    delay::Delay,
    dma::{Dma, DmaPriority},
    dma_buffers, dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    peripherals::{self, Peripherals},
    prelude::*,
    rng::Rng,
    timer::timg::TimerGroup,
};

#[embassy_executor::task]
pub async fn synth_task() {
    let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, 128);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.i2s0channel;

        let i2s = I2s::new(
            peripherals.I2S0,
            Standard::Philips,
            DataFormat::Data16Channel16,
            22050.Hz(),
            dma_channel.configure(false, DmaPriority::Priority0),
            rx_descriptors,
            tx_descriptors,
        );

        let mut i2s_tx = i2s
            .i2s_tx
            .with_bclk(peripherals.GPIO16)
            .with_ws(peripherals.GPIO17)
            .with_dout(peripherals.GPIO15)
            .build();

        let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
        loop {
            Timer::after(Duration::from_millis(100)).await;
            esp_println::println!("{:?}", transfer.available());
        }
    }
}

#[embassy_executor::task]
async fn intro_text() {
    esp_println::println!("-:=  Little Squeaky Machine V1.0  =:-");
    esp_println::println!("-:=     (Little Weirdo Engine)    =:-");
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("-:=                               =:-");
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut rng = Rng::new(peripherals.RNG);
    esp_println::println!("-:= random :: {:?}", rng.random());
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    spawner.spawn(intro_text()).ok();
    spawner.spawn(synth_task()).ok();
    loop {
        Timer::after(Duration::from_millis(100)).await;
    }
}
