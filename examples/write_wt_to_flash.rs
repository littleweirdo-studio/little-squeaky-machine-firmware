#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

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

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // init CPU
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());

    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(timg0.timer0);

    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);

     let mut storage;
      unsafe {
        let peripherals: esp_hal::peripherals::Peripherals = esp_hal::peripherals::Peripherals::steal();
        storage = FlashStorage::new(peripherals.FLASH);
      }
    println!("Flash size = {} MB", storage.capacity() / 1_000_000);
    println!("Writing to flash");
    let _ = storage.write(
        0x00220000,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav0.raw"),
    );
    let _ = storage.write(
        0x00220000 + 1 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav1.raw"),
    );
    let _ = storage.write(
        0x00220000 + 2 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav2.raw"),
    );
    let _ = storage.write(
        0x00220000 + 3 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav3.raw"),
    );
    let _ = storage.write(
        0x00220000 + 4 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav4.raw"),
    );
    let _ = storage.write(
        0x00220000 + 5 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav5.raw"),
    );
    let _ = storage.write(
        0x00220000 + 6 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav6.raw"),
    );
    let _ = storage.write(
        0x00220000 + 7 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav7.raw"),
    );
    let _ = storage.write(
        0x00220000 + 8 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav8.raw"),
    );
    let _ = storage.write(
        0x00220000 + 9 * 1200,
        include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav9.raw"),
    );

    let _ = storage.write(
        0x00220000 + 10 * 1200,
        include_bytes!("../src/soundbank/soundbank_samples/src/wav0.raw"),
    );

    let _ = storage.write(
        0x00220000 + 10 * 1200 + 16458,
        include_bytes!("../src/soundbank/soundbank_samples/src/wav1.raw"),
    );

    println!("Writing to flash done");
}
