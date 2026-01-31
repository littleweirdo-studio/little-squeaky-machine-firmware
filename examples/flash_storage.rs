#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use core::ops::Range;
use core::str::Chars;

use alloc::string::String;
use embassy_executor::Spawner;
extern crate alloc;

use embedded_storage::nor_flash::NorFlash;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::psram::{self, PsramSize};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::{AnyTimer, timg::TimerGroup};
use esp_println::println;
use esp_storage::FlashStorage;

use embedded_storage::{ReadStorage, Storage};
use little_weirdo::fs;

const MAP_FLASH_RANGE: Range<u32> = 0x00220000..(0x00220000 + 262144);

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
   
    // init CPU
    let mut config = esp_hal::Config::default();
    let peripherals = esp_hal::init(config);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(timg0.timer0);

    // Use 64kB in dram2_seg for the heap, which is otherwise unused.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 64000);

    // Initialize PSRAM and add it as a heap memory region
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let storage;
    unsafe {
        let peripherals: esp_hal::peripherals::Peripherals =
            esp_hal::peripherals::Peripherals::steal();
        storage = FlashStorage::new(peripherals.FLASH);
    }
   
    let mut data_buffer: alloc::boxed::Box<[u8; 2042]> = alloc::boxed::Box::new([0u8; 2042]);

    esp_println::println!("-:= > \n{}-:= >", esp_alloc::HEAP.stats());

    let mut filesystem: fs::WeirdoFileSystem<FlashStorage>  = fs::WeirdoFileSystem::new(storage, 0x00220000, 0x100000);

    for i in 0..5 {
  let len =  filesystem.read_key_value(&[0,i],&mut *data_buffer).unwrap();
    esp_println::println!("{:?}", len);
     let patch: little_weirdo::synth::patch::Patch = postcard::from_bytes(&mut *data_buffer).unwrap();
    esp_println::println!("{:?}", patch.name);

    }

    esp_println::println!("-:= > \n{}-:= >", esp_alloc::HEAP.stats());

  

}
