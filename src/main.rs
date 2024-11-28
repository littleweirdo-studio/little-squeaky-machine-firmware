#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use esp_backtrace as _;
use esp_hal::{analog::dac::Dac, delay::Delay, dma::{Dma, DmaPriority}, dma_buffers, dma_circular_buffers, i2s::master::{DataFormat, I2s, Standard}, peripherals::{self, Peripherals}, prelude::*, rng::Rng};
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel as Chn, signal::Signal,
};




#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut rng = Rng::new(peripherals.RNG);
    esp_println::println!("{:?}", rng.random());
    spawner.spawn(hello()).ok();
    spawner.spawn(synth_task()).ok();
}

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
            dma_channel.configure(
                false,
                DmaPriority::Priority0,
            ),
            rx_descriptors,
            tx_descriptors,
        ).into_async();
        
    
        let mut i2s_tx = i2s.i2s_tx
        .with_bclk(peripherals.GPIO1)
        .with_ws(peripherals.GPIO2)
        .with_dout(peripherals.GPIO15)
        .build();

        
        let mut transfer = i2s_tx.write_dma_circular( tx_buffer).unwrap();   
        loop {
           
        }
    }    
}


  

#[embassy_executor::task]
async fn hello() {
    esp_println::println!("-:=                               =:-");
    esp_println::println!("-:=  Little Squeaky Machine V1.0  =:-");
    esp_println::println!("-:=     (Little Weirdo Engine)    =:-");
  
    loop {
        Delay::new().delay_millis(100);
    }
}