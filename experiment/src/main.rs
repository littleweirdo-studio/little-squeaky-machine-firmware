#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

extern crate alloc;

//
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock, ram,
    system::{Cpu},
};
use esp_hal::timer::timg::TimerGroup;
//

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    esp_println::println!("⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⢎⡑⡎⢱⡇⢸⣏⡉⣎⣱⣇⠜⢇⢸ ⡷⢾⣎⣱⡎⠑⣇⣸⡇⡷⣸⣏⡉");
    esp_println::println!("⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠢⠜⠣⠪⠣⠜⠧⠤⠇⠸⠇⠱ ⠇ ⠇⠸⠇⠸⠣⠔⠇⠸⠇⠇⠹⠧⠤");
    esp_println::println!("Experiment #1\n");
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    esp_println::println!("-:= > Set CPU Speed to {:?}", config.cpu_clock());
    esp_println::println!(
        "-:= > Starting Little Squeaky Machine on core {}",
        Cpu::current() as usize
    );
    let peripherals = esp_hal::init(config);

    // Use 64kB in dram2_seg for the heap, which is otherwise unused.
    heap_allocator!(#[ram(reclaimed)] size: 64000);

    // Initialize PSRAM and add it as a heap memory region
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(timg0.timer0);

    loop {
        Timer::after(Duration::from_millis(10)).await;
    }
}
