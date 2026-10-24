#![no_std]
#![no_main]

mod led;
use core::ptr::addr_of_mut;

use crate::led::led_task;

mod codec;
use crate::codec::codec_task;

mod synth;
use synth::synth_loop;

mod midi_uart;
use midi_uart::midi_uart_task;

mod midi_usb;
use midi_usb::midi_usb_task;

mod oled;
use crate::oled::oled_task;

mod encoder;
use crate::encoder::encoder_task;

mod keyboard_input;
use crate::keyboard_input::keyboard_input_task;

mod sequence;

mod mapping;
use crate::mapping::mapping_task;

mod touch_input;
use crate::touch_input::touch_input_task;

mod tilt;
use crate::tilt::tilt_task;

//
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    channel::Channel,
    mutex::Mutex,
    pubsub::PubSubChannel,
};
use embassy_time::{Duration, Timer};
use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::{
    Async,
    clock::CpuClock,
    ram,
    system::{Cpu, CpuControl, Stack},
};
use static_cell::StaticCell;
//

extern crate alloc;

//
use esp_hal::timer::timg::TimerGroup;
//

// channel for sharing midi message between tasks and processes running on different cores.
static MIDI_MESSAGE_CHANNEL: PubSubChannel<CriticalSectionRawMutex, midi_msg::MidiMsg, 50, 5, 5> =
    PubSubChannel::new();

static MIDI_OUT_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, midi_msg::MidiMsg, 50> =
    Channel::new();

// create stack for second core (running the Synth tasks, 220KB stack reserved)
static mut APP_CORE_STACK: Stack<250_000> = Stack::new();

const USB_MIDI_ENABLED: bool = true;
const UART_MIDI_ENABLED: bool = false;
const TILT_ENABLED: bool = false;
const TOUCH_ENABLED: bool = false;
const MIX_INPUT: bool = true;

type I2c1Bus = Mutex<NoopRawMutex, esp_hal::i2c::master::I2c<'static, Async>>;

esp_bootloader_esp_idf::esp_app_desc!();

const VERSION: u8 = 1;

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    esp_println::println!("⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⢎⡑⡎⢱⡇⢸⣏⡉⣎⣱⣇⠜⢇⢸ ⡷⢾⣎⣱⡎⠑⣇⣸⡇⡷⣸⣏⡉");
    esp_println::println!("⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠢⠜⠣⠪⠣⠜⠧⠤⠇⠸⠇⠱ ⠇ ⠇⠸⠇⠸⠣⠔⠇⠸⠇⠇⠹⠧⠤");
    esp_println::println!("Firmware {}\n", VERSION);
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    esp_println::println!("-:= > Set CPU Speed to {:?}", config.clock_config());
    esp_println::println!(
        "-:= > Starting Little Squeaky Machine on core {}",
        Cpu::current() as usize
    );
    let peripherals = esp_hal::init(config);

    let config =
        esp_hal::i2c::master::Config::default().with_frequency(esp_hal::time::Rate::from_khz(400));
    let i2c = esp_hal::i2c::master::I2c::new(peripherals.I2C0, config)
        .unwrap()
        .with_sda(peripherals.GPIO47)
        .with_scl(peripherals.GPIO48)
        .into_async();

    static I2C_BUS: StaticCell<I2c1Bus> = StaticCell::new();
    let i2c_bus = I2C_BUS.init(Mutex::new(i2c));

    // Use 64kB in dram2_seg for the heap, which is otherwise unused.
    heap_allocator!(#[ram(reclaimed)] size: 64000);

    // Initialize PSRAM and add it as a heap memory region
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    // spawn the tasks
    spawner.spawn(led_task().unwrap());
    Timer::after(Duration::from_millis(10)).await;

    let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let _guard = cpu_control
        .start_app_core(unsafe { &mut *addr_of_mut!(APP_CORE_STACK) }, move || {
            synth_loop();
        })
        .unwrap();

    // Create MIDI USB device
    if USB_MIDI_ENABLED {
        Timer::after(Duration::from_millis(10)).await;
        spawner.spawn(midi_usb_task().unwrap());
    }

    if UART_MIDI_ENABLED {
        Timer::after(Duration::from_millis(10)).await;
        spawner.spawn(midi_uart_task().unwrap());
    }

    Timer::after(Duration::from_millis(10)).await;
    spawner.spawn(oled_task(i2c_bus).unwrap());
    Timer::after(Duration::from_millis(10)).await;

    spawner.spawn(codec_task(i2c_bus).unwrap());

    Timer::after(Duration::from_millis(10)).await;
    spawner.spawn(keyboard_input_task().unwrap());
    spawner.spawn(encoder_task().unwrap());
    spawner.spawn(mapping_task().unwrap());
    
    if TOUCH_ENABLED {
        spawner.spawn(touch_input_task(i2c_bus).unwrap());
    }

    if TILT_ENABLED {
        spawner.spawn(tilt_task(i2c_bus).unwrap());
    }

    loop {
        Timer::after(Duration::from_millis(10)).await;
    }
}
