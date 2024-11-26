#![no_std]
#![no_main]

use embedded_storage::{ReadStorage, Storage};
use esp32_hal as hal;

use hal::prelude::*;

use esp_storage::FlashStorage;

use esp_backtrace as _;
use esp_println::println;

#[entry]
fn main() -> ! {
    let mut bytes = [0u8; 32];

    let mut flash = FlashStorage::new();

    let flash_addr = 0x290000;
    println!("Flash size = {}", flash.capacity());

    flash.read(flash_addr, &mut bytes).unwrap();
    println!("Read from {:x}:  {:02x?}", flash_addr, &bytes[..32]);

    flash.write(flash_addr, &bytes).unwrap();
    println!("Written to {:x}: {:02x?}", flash_addr, &bytes[..32]);

    let mut reread_bytes = [0u8; 32];
    flash.read(flash_addr, &mut reread_bytes).unwrap();
    println!("Read from {:x}:  {:02x?}", flash_addr, &reread_bytes[..32]);

    loop {}
}