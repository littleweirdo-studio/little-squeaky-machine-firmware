#![no_std]
#![no_main]

use esp32_hal::{
    clock::ClockControl, 
    peripherals::{Peripherals, self}, 
    prelude::*, 
    Rtc, 
    dac, 
    IO, 
    Delay, 
    timer::{TimerGroup, Timer0}, 
    Timer, 
    interrupt::{self, Priority},
    cpu_control::{CpuControl, Stack},
    };
use esp_backtrace as _;
use esp_println::{self, println};
use little_weirdo::synth;
use little_weirdo::synth::envelope::Envelop;
use little_weirdo::synth::oscillator::Waveform;
use little_weirdo::synth::patch::Patch;
use little_weirdo::synth::filter::FilterConfig;
const DELAY_US:u64 = 44;
const DELAY_OFFSET_US:u64  = 5;
static mut APP_CORE_STACK: Stack<8192> = Stack::new();


#[entry]
fn main() -> ! {

    let  patch = Patch {
        voice_1: Waveform::SawTooth,
        voice_1_mix_level: 100,
        voice_2: Waveform::Square10,
        voice_2_mix_level: 80,
        lfo_1: 10,
        lfo_1_mix_level: 0,
        voice_1_env: Envelop {
            attack_time: 1000,
            decay_time: 1000,
            release_time: 1000,
            sustain_level: 50,
        },
        voice_2_env: Envelop {
            attack_time: 1000,
            decay_time: 1000,
            release_time: 1000,
            sustain_level: 50,
        },

        voice_1_detune:     0,
        voice_2_detune:     4,
        filter_config: FilterConfig {
            cutoff_frequency: 100,
            filter_on: true
        },
        main_gain: 70,
    };

    let mut synth: synth::Synth = synth::Synth::new(22500, patch);

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let rtc = Rtc::new(peripherals.RTC_CNTL);


    //utalize second core
    let mut cpu_control = CpuControl::new(system.cpu_control);
    let cpu1_fnctn = || {
        cpu1_task();
    };
    let _guard = cpu_control
        .start_app_core(unsafe { &mut APP_CORE_STACK }, cpu1_fnctn)
        .unwrap();

    

    // Create DAC instances

    let mut io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pin25 = io.pins.gpio25.into_analog();
    let analog = peripherals.SENS.split();
    let mut dac1 = dac::DAC1::dac(analog.dac1, pin25).unwrap();
    let mut delay = Delay::new(&clocks);
    
    let mut sum = 0;
    let mut overrun = 0;
    let mut high = 0;
    let mut low = DELAY_US;
    let mut moment: i32 = 0;
    
    let mut start_note: u8 = 30;

    loop {

        start_note += 1;
        if start_note > 100 { start_note = 30 };
    synth.note_on(0,start_note);
    let start1_time = rtc.get_time_ms();
    for n in 0..22500 {

        let start_time = rtc.get_time_us();
        let output:i16 = synth.clock_and_output();
       // let down_sample:u8 = (((output / 2.0) * 255.0 )) as u8;
       // dac1.write(down_sample);
        let stop_time = rtc.get_time_us();
        let calculation_cost = stop_time - start_time;
        sum += calculation_cost;
        if calculation_cost < DELAY_US {
            if low > calculation_cost { low = calculation_cost};
            delay.delay_us((DELAY_US - calculation_cost - DELAY_OFFSET_US) as u32);
        } else {
            overrun += 1;
            if high < calculation_cost { 
                high = calculation_cost;
                moment = n;
            };
            
        }
    }
    let stop1_time = rtc.get_time_ms();
    println!("----- 1 sec cycle @ 22500Hz (max 44µs ) -----");
    println!("> average - total compute time spend  {}", sum);
    println!("> clock :: average {}µs", sum / 22500);
    println!("> deadline passed {}x,",  overrun);
    println!("> highest process time {}µs (@{})",  high, moment);
    println!("> lowest  process time {}µs",  low);
    println!("> headroom = {} %", (1_000_000 - sum) / 10000);
    println!("----------------------------------------------");
    sum = 0;
    overrun = 0;
    high = 0;
    low = DELAY_US;
    }
}

fn cpu1_task() -> ! {
    loop {
        
    }
}