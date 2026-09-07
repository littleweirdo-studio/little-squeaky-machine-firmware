#![no_std]
#![no_main]

extern crate alloc;


//
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, ram, system::Cpu};
//
use esp_hal::{
    gpio::{Input, InputConfig, Output, OutputConfig, Pull},
    peripherals::Peripherals,
};

use embassy_usb::Builder;
use esp_hal::otg_fs::Usb;
use esp_hal::otg_fs::asynch::Driver;

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

    esp_alloc::heap_allocator!(#[ram(reclaimed)] size: 64 * 1024);
    esp_alloc::heap_allocator!(size: 36 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    

    // Buffer for USB OUT endpoint
    let mut ep_out_buffer = [0u8; 1024];

    // Configure USB driver
    let config = esp_hal::otg_fs::asynch::Config::default();
    let driver = Driver::new(usb, &mut ep_out_buffer, config);
 // Setup USB device configuration

    let mut config = embassy_usb::Config::new(0x303A, 0x3001); // Vendor ID and Product ID
    config.manufacturer = Some("Hi Squeaky Things"); // Device manufacturer
    config.product = Some("Little Squeaky Machine"); // Device product name
    config.serial_number = Some("LSM0000001"); // Device serial number
    config.device_class = 0x00; // USB device class
    config.device_sub_class = 0x00; // USB device sub-class
    config.device_protocol = 0x00; // USB device protocol
    config.composite_with_iads = false; // Not using IADs

    // Buffers for USB descriptors and control
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    // Create USB device builder
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );


    let mut audio_class = AudioSource::new(&mut builder, &[44100], SampleWidth::Width2Byte, 40,  None  );

    
 // Build the USB device
    let mut usb_device = builder.build();

    // Run the USB device in background
    let usb_future = usb_device.run();

    loop {
        Timer::after(Duration::from_millis(10)).await;
    }
}
