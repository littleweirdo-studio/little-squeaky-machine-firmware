#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

extern crate alloc;
use oled_tasks::oled_task;
use synth_tasks::synth_loop;
use core::ptr::addr_of_mut;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;
use embassy_usb::Builder;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::cpu_control::{CpuControl, Stack};
use esp_hal::otg_fs::asynch::Driver;
use esp_hal::otg_fs::Usb;
use esp_hal::{
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};
use esp_hal::psram;
use esp_hal_embassy::Executor;
use usbd_midi::data::usb_midi::midi_packet_reader::MidiPacketBufferReader;

use static_cell::StaticCell;
//allocations

static SHARED: Channel<CriticalSectionRawMutex, usbd_midi::data::midi::message::Message, 25> =
    Channel::new();
static mut APP_CORE_STACK: Stack<128_000> = Stack::new();

mod synth_tasks;
mod oled_tasks;



#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    esp_println::println!("-:=  LttL Squeaky Machine v1.1       =:-");
    esp_println::println!("-:=  Powered by LttL Weirdɵ v0.1     =:-");
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    esp_println::println!(
        "> Set CPU Speed to {:?} MHz\n",
        config.cpu_clock.frequency().to_MHz()
    );
    let peripherals = esp_hal::init(config);
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);

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
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                let _ = spawner.spawn(synth_loop()).ok();
            });
        })
        .unwrap();

    //

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    let mut ep_out_buffer = [0u8; 1024];
    let config = esp_hal::otg_fs::asynch::Config::default();

    let driver = Driver::new(usb, &mut ep_out_buffer, config);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0x303A, 0x3001);
    config.manufacturer = Some("Hi Squeaky Things");
    config.product = Some("Lttl Squeak Machine");
    config.serial_number = Some("00110022");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
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
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    let mut midi = MidiClass::new(&mut builder, 1, 1, 64);
    let mut usb = builder.build();
    let usb_fut = usb.run();
   // println!("wait for synth");
    Timer::after(Duration::from_millis(1000)).await;
   // println!("start oled");
    let proces_midi = async {
        loop {
            midi.wait_connection().await;
            let _ = midi_read(&mut midi).await;
        }
    };
    let _ = spawner.spawn(oled_task());
    Timer::after(Duration::from_millis(1000)).await;
   // println!("midi");
    join(usb_fut, proces_midi).await;

    //
}

async fn midi_read<'d>(class: &mut MidiClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let buffer_reader = MidiPacketBufferReader::new(&buf, n);
        for packet in buffer_reader.into_iter() {
            if let Ok(packet) = packet {
                match packet.message {
                    _ => {
                        SHARED.send(packet.message).await;
                    }
                }
            }
        }
    }
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}
