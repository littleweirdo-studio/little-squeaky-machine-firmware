#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
//
mod led;
mod midi_uart;
mod midi_usb;
mod oled;
mod potmeter;
mod synth;
use crate::led::led_task;
use crate::midi_uart::midi_uart_task;
use crate::midi_usb::midi_read;
use crate::oled::oled_task;
use crate::potmeter::potmeter_task;
use crate::synth::synth_loop;
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
use esp_hal::psram;
use esp_hal::timer::{timg::TimerGroup, AnyTimer};
//

// channel for sharing midi message between tasks and processes running on different cores.
static MIDI_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, usbd_midi::message::Message, 30> =
    Channel::new();

// create stack for second core (running the Synth tasks, 160KB stack reserved)
static mut APP_CORE_STACK: Stack<220_000> = Stack::new();

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

    let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let _guard = cpu_control
        .start_app_core(unsafe { &mut *addr_of_mut!(APP_CORE_STACK) }, move || {
            synth_loop();
        })
        .unwrap();

    // Create MIDI USB device
    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    let mut ep_out_buffer = [0u8; 1024];
    let config = esp_hal::otg_fs::asynch::Config::default();
    let driver = Driver::new(usb, &mut ep_out_buffer, config);
    // Setup the configuration for the USB device
    let mut config = embassy_usb::Config::new(0x303A, 0x3001);
    config.manufacturer = Some("Hi Squeaky Things");
    config.product = Some("Lttl Squeak Machine");
    config.serial_number = Some("00110022");
    config.device_class = 0x00;
    config.device_sub_class = 0x00;
    config.device_protocol = 0x00;
    config.composite_with_iads = false;
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    // initlize the actual MIDI class
    let mut midi_class = MidiClass::new(&mut builder, 1, 1, 64);
    let mut usb_device = builder.build();
    let usb_future = usb_device.run();
    Timer::after(Duration::from_millis(100)).await;
    let process_midi = async {
        loop {
            midi_class.wait_connection().await;
            let _ = midi_read(&mut midi_class).await;
        }
    };
    // spawn the tasks
    let _ = spawner.spawn(led_task());
    Timer::after(Duration::from_millis(100)).await;
    let _ = spawner.spawn(oled_task());
    // let _ = spawner.spawn(midi_uart_task());
    //  let _ = spawner.spawn(potmeter_task());
    Timer::after(Duration::from_millis(100)).await;
    // spawn the USB device task (MIDI)
    join(usb_future, process_midi).await;
}
