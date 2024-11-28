#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{analog::dac::Dac, delay::Delay, peripherals::{self, Peripherals}, prelude::*, rng::Rng};
use embassy_executor::Spawner;
use little_weirdo::synth::{self, patch::Patch};

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("Init!");
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut rng = Rng::new(peripherals.RNG);
    esp_println::println!("{:?}", rng.random());
    spawner.spawn(writer()).ok();
}

#[embassy_executor::task]
async fn writer() {
    let mut dac;
    unsafe {
        let peripherals = Peripherals::steal();
        let dac1_pin = peripherals.GPIO25;
        dac = Dac::new(peripherals.DAC1, dac1_pin);
    }
    esp_println::println!("-:=                               =:-");
    esp_println::println!("-:=  Little Squeaky Machine V1.0  =:-");
    esp_println::println!("-:=     (Little Weirdo Engine)    =:-");
    let mut synth = synth::Synth::new(22500, Patch::default());
    synth.note_on(0, 60, 127);
    
    loop {
       let sample = synth.clock_and_output();
        Delay::new().delay_millis(100);
    }
}