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

mod potmeter;
use crate::potmeter::potmeter_task;

mod touch_input;
use crate::touch_input::touch_input_task;

//
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use esp_alloc::{heap_allocator, psram_allocator};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    psram, ram,
    system::{Cpu, CpuControl, Stack},
};
//

extern crate alloc;

//
use esp_hal::timer::timg::TimerGroup;
//

// channel for sharing midi message between tasks and processes running on different cores.
static MIDI_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, midi_msg::MidiMsg, 30> =
    Channel::new();

static MIDI_OUT_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, midi_msg::MidiMsg, 30> =
    Channel::new();

// create stack for second core (running the Synth tasks, 220KB stack reserved)
static mut APP_CORE_STACK: Stack<250_000> = Stack::new();

const USB_MIDI_ENABLED: bool = true;
const UART_MIDI_ENABLED: bool = false;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    esp_println::println!("⡇ ⡇⢹⠁⢹⠁⡇ ⣏⡉ ⢎⡑⡎⢱⡇⢸⣏⡉⣎⣱⣇⠜⢇⢸ ⡷⢾⣎⣱⡎⠑⣇⣸⡇⡷⣸⣏⡉");
    esp_println::println!("⠧⠤⠇⠸ ⠸ ⠧⠤⠧⠤ ⠢⠜⠣⠪⠣⠜⠧⠤⠇⠸⠇⠱ ⠇ ⠇⠸⠇⠸⠣⠔⠇⠸⠇⠇⠹⠧⠤");
    esp_println::println!("Firmware v0.1.0 with Little Weirdo v0.1.1\n");
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

    // spawn the tasks
    let _ = spawner.spawn(led_task());
    Timer::after(Duration::from_millis(10)).await;

    let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let _guard = cpu_control
        .start_app_core(unsafe { &mut *addr_of_mut!(APP_CORE_STACK) }, move || {
            synth_loop();
        })
        .unwrap();

    Timer::after(Duration::from_millis(500)).await;

    // Create MIDI USB device
    if USB_MIDI_ENABLED {
        Timer::after(Duration::from_millis(10)).await;
        let _ = spawner.spawn(midi_usb_task());
    }

    if UART_MIDI_ENABLED {
        Timer::after(Duration::from_millis(10)).await;
        let _ = spawner.spawn(midi_uart_task());
    }

    Timer::after(Duration::from_millis(10)).await;
    let _ = spawner.spawn(oled_task());

    Timer::after(Duration::from_millis(10)).await;
    let _ = spawner.spawn(potmeter_task());

    Timer::after(Duration::from_millis(10)).await;
    let _ = spawner.spawn(codec_task());

    Timer::after(Duration::from_millis(10)).await;
    let _ = spawner.spawn(touch_input_task());

    loop {
        Timer::after(Duration::from_millis(10)).await;
        //     Timer::after(Duration::from_millis(10)).await;
        //    MIDI_MESSAGE_CHANNEL.send(midi_msg::MidiMsg::ChannelVoice { channel: midi_msg::Channel::Ch1, msg: midi_msg::ChannelVoiceMsg::NoteOn { note: 60, velocity: 100} }).await;
        //     Timer::after(Duration::from_millis(10)).await;
        //     MIDI_MESSAGE_CHANNEL.send(midi_msg::MidiMsg::ChannelVoice { channel: midi_msg::Channel::Ch1, msg: midi_msg::ChannelVoiceMsg::NoteOff { note: 60, velocity: 100} }).await;
    }
}
