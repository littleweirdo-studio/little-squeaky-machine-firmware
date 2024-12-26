#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
use embassy_executor::Spawner;
use esp_backtrace as _;

use esp_hal::rtc_cntl::Rtc;
use esp_hal::{
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};
use little_weirdo::synth;
use little_weirdo_soundbanks::soundbanks::SOUND_BANK_PURE_ELEKTRO;

const SAMPLE_RATE: u32 = 44100;
const DELAY_US: u32 = 10000000 / SAMPLE_RATE;
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // init CPU
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    esp_hal_embassy::init([timer1, timer2]);

    let mut synth: synth::Synth =
        synth::Synth::new(SAMPLE_RATE as u16, &SOUND_BANK_PURE_ELEKTRO.patches[0], &SOUND_BANK_PURE_ELEKTRO);
  
    let mut sum = 0;
    let mut overrun = 0;
    let mut high = 0;
    let mut low: i64 = DELAY_US as i64;
    let mut moment: u32 = 0;
    for _x in 0..5 {
        let start_time = rtc.current_time().and_utc().timestamp_micros();
        synth.load_patch(&SOUND_BANK_PURE_ELEKTRO.patches[1]);
        let stop_time = rtc.current_time().and_utc().timestamp_micros();
        esp_println::println!("> patch change in µs (max 22µs) {}", stop_time - start_time);
    }
    for _x in 0..5 {
        synth.note_on(60, 127);
        for n in 0..SAMPLE_RATE {
            let start_time = rtc.current_time().and_utc().timestamp_micros();
            let _output = synth.clock_and_output();
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
