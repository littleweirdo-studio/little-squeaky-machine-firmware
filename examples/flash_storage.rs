#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]


use core::ops::Range;

use embassy_executor::Spawner;
extern crate alloc;

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::psram::{self, PsramSize};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::{AnyTimer, timg::TimerGroup};
use esp_println::println;
use esp_storage::FlashStorage;

use embedded_storage::{ReadStorage, Storage};

const MAP_FLASH_RANGE: Range<u32> = 0x00220000..(0x00220000+0x800000);


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




    let mut storage;
    unsafe {
        let peripherals: esp_hal::peripherals::Peripherals =
            esp_hal::peripherals::Peripherals::steal();
        storage = FlashStorage::new(peripherals.FLASH);
    }
  let storage = embassy_embedded_hal::adapter::BlockingAsync::new(storage);

   

    let flash_range = 0x00220000..0x00320000;
    let mut data_buffer: alloc::boxed::Box<[u8;500_000]>  = alloc::boxed::Box::new([0u8; 500_000]);
    
      esp_println::println!("-:= > \n{}-:= >", esp_alloc::HEAP.stats());
      

     let mut map_storage = sequential_storage::map::MapStorage::new(
        storage,
        const { sequential_storage::map::MapConfig::new(MAP_FLASH_RANGE) },
        sequential_storage::cache::KeyPointerCache:: <4,u16,8> ::new(),
    );
  // let _ = map_storage.erase_all().await;
    
    println!("data = {:?}", include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav0.raw"));
    println!("store item");
    let _ = map_storage.store_item(&mut *data_buffer, &1_001u16, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav0.raw")).await.unwrap();
    println!("store item done");
     println!("store item fetch");
     let result =  map_storage.fetch_item::<&[u8]>(&mut *data_buffer, &1_001u16).await.unwrap();
        println!("store item fetch done");
    println!("{:?}", result);


}
