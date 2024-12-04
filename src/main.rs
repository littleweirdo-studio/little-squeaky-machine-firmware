#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaPriority},
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    peripherals::{self, Peripherals},
    prelude::*,
    rng::Rng,
    timer::timg::TimerGroup,
};

use little_weirdo::synth;
use little_weirdo::synth::patch::Patch;

#[embassy_executor::task]
pub async fn synth_task() {
    const SAMPLE_RATE: u32 = 22050;
    const BUFFER_SIZE: usize = 4 * 110;
       
    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE as u16, Patch::default());

    let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, BUFFER_SIZE * 2);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.i2s0channel;

        let i2s = I2s::new(
            peripherals.I2S0,
            Standard::Philips,
            DataFormat::Data16Channel16,
            SAMPLE_RATE.Hz(),
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

        //wait 1 seconds
        Timer::after(Duration::from_secs(1)).await;
        // play one note
        synth.note_on(0, 60, 100);
       
        /// Double buffering (110 samples ~  5ms latency), not realtime, need revisioning. 
        let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            let space_available_in_buffer= transfer.available().unwrap();
            if space_available_in_buffer > 0 {
                let avail = usize::min(BUFFER_SIZE, space_available_in_buffer);
                for i in 0..(avail / 4) {
                    let output: i16 = synth.clock_and_output();
                    let bytes = output.to_be_bytes();
                    buffer[i * 4] = bytes[1];
                    buffer[i * 4 + 1] = bytes[0];
                    buffer[i * 4 + 2] = bytes[1];
                    buffer[i * 4 + 3] = bytes[0];
                }
                transfer.push(&buffer[0..avail]).unwrap();
            };
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
