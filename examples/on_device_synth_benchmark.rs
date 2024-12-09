#![no_std]
#![no_main]

use core::ptr::addr_of_mut;

use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::cpu_control::{CpuControl, Stack};
use esp_hal::otg_fs::Usb;
use esp_hal::rtc_cntl::Rtc;
//use esp_hal::touch::{Touch, TouchPad};
use esp_hal::Cpu;
use esp_hal::{
    dma::{Dma, DmaPriority},
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    otg_fs::UsbBus,
    peripherals::Peripherals,
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};

use little_weirdo::synth::effects::overdrive::{KindOfOverdrive, Overdrive};
use usb_device::{
    device::StringDescriptors,
    prelude::{UsbDeviceBuilder, UsbVidPid},
};
use usbd_midi::data::byte::u7::U7;
use usbd_midi::data::midi::message::Message;
use usbd_midi::{
    data::usb_midi::midi_packet_reader::MidiPacketBufferReader, midi_device::MidiClass,
};
static mut EP_MEMORY: [u32; 1024] = [0; 1024];

use esp_hal_embassy::Executor;
use little_weirdo::synth::{self, patches::Patches};

static SHARED: Channel<CriticalSectionRawMutex, usbd_midi::data::midi::message::Message, 20> =
    Channel::new();

static mut APP_CORE_STACK: Stack<32000> = Stack::new();
use static_cell::StaticCell;

const SAMPLE_RATE: u32 = 44100;
const DELAY_US: u32 = 1000000 / SAMPLE_RATE;
#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    esp_hal_embassy::init([timer1, timer2]);


    let mut synth: synth::Synth = synth::Synth::new(
        SAMPLE_RATE as u16,
        Patches::get_patch(Patches::BassGuitar),
    );

    synth.filter.filter_on = true;

    let mut sum = 0;
    let mut overrun = 0;
    let mut high = 0;
    let mut low: i64 = DELAY_US as i64;
    let mut moment: u32 = 0;

    let mut start_note: u8 = 60;

    for x in 0..5 {
        synth.note_on(0, start_note, 127);
        for n in 0..SAMPLE_RATE {
            let start_time = rtc.current_time().and_utc().timestamp_micros();
            let _output: i16 = synth.clock_and_output();
            let stop_time = rtc.current_time().and_utc().timestamp_micros();
            let calculation_cost = stop_time - start_time;
            sum += calculation_cost;
            if calculation_cost < DELAY_US as i64 {
                if low > calculation_cost {
                    low = calculation_cost
                };
            } else {
                overrun += 1;
                if high < calculation_cost {
                    high = calculation_cost;
                    moment = n;
                };
            }
        }
         esp_println::println!("----- 1 sec cycle @ 44100Hz (max 22µs ) -----");
         esp_println::println!("> average - total compute time spend  {}", sum);
         esp_println::println!("> clock :: average {}µs", sum / SAMPLE_RATE as i64);
         esp_println::println!("> deadline passed {}x,", overrun);
         esp_println::println!("> highest process time {}µs (@{})", high, moment);
         esp_println::println!("> lowest  process time {}µs", low);
         esp_println::println!("> headroom = {} %", (1_000_000 - sum) / 10000);
         esp_println::println!("----------------------------------------------");
        sum = 0;
        overrun = 0;
        high = 0;
        low = DELAY_US as i64;
    }
}
