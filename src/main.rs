#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
//
mod led;
mod midi_usb;
mod midi_uart;
mod oled;
mod potmeter;
mod synth;
mod codec;
mod touch_input;

use crate::led::led_task;
use crate::oled::oled_task;
use crate::potmeter::potmeter_task;
use crate::synth::synth_loop;
use crate::codec::codec_task;
use crate::touch_input::touch_input_task;

//
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embassy_usb::class::midi::MidiClass;
use embassy_usb::Builder;
use esp_hal::peripheral::Peripheral;
use esp_hal::peripherals::PSRAM;
use midi_uart::midi_uart_task;
use midi_usb::{midi_usb_task};
use smart_leds::colors;
use usbd_midi::message::U7;
use usbd_midi::Message;
//
extern crate alloc;
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;
use esp_alloc as _;
use esp_backtrace as _;
//
use esp_hal::cpu_control::{CpuControl, Stack};
use esp_hal::otg_fs::asynch::Driver;
use esp_hal::otg_fs::Usb;
use esp_hal::{psram, uart};
use esp_hal::timer::{timg::TimerGroup, AnyTimer};
//

// channel for sharing midi message between tasks and processes running on different cores.
static MIDI_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, usbd_midi::message::Message, 30> =
    Channel::new();

static MIDI_OUT_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, usbd_midi::message::Message, 30> =
    Channel::new();


// create stack for second core (running the Synth tasks, 220KB stack reserved)
static mut APP_CORE_STACK: Stack<220_000> = Stack::new();

const usb_midi_enabled: bool = false;
const uart_midi_enabled: bool = true;


#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    esp_println::println!("-:=  LttL Squeaky Machine v1.1       =:-");
    esp_println::println!("-:=  Powered by LttL Weirdɵ v0.1     =:-");
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::_240MHz);
    esp_println::println!("> Set CPU Speed to {:?} MHz\n", config.cpu_clock);
    let peripherals = esp_hal::init(config);
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);
    esp_println::println!("-:= > Heap free {} KB", esp_alloc::HEAP.free() / 1_000);

    // TODO: do we need 4 timers?
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer_group_1 = TimerGroup::new(peripherals.TIMG1);

    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    let timer3: AnyTimer = timer_group_1.timer0.into();
    let timer4: AnyTimer = timer_group_1.timer1.into();

    esp_hal_embassy::init([timer1, timer2, timer3, timer4]);
    // spawn the tasks
    let _ = spawner.spawn(led_task());
    Timer::after(Duration::from_millis(10)).await;
  
    
    let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let _guard = cpu_control
        .start_app_core(unsafe { &mut *addr_of_mut!(APP_CORE_STACK) }, move || {
            synth_loop();
        })
        .unwrap();


    Timer::after(Duration::from_millis(100)).await;
  
    // Create MIDI USB device
    if usb_midi_enabled {
        Timer::after(Duration::from_millis(10)).await;
        spawner.spawn(midi_usb_task());
    }
    

    if uart_midi_enabled {
        Timer::after(Duration::from_millis(10)).await;
        spawner.spawn(midi_uart_task());
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
        Timer::after(Duration::from_millis(500)).await;
    }

}
