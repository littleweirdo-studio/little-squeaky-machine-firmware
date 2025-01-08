#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::psram;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::{
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};
use esp_println::println;
use little_weirdo::synth;
use little_weirdo::synth::data::wavetables::{BoxedWavetable, BoxedWavetables};
use little_weirdo::synth::patch::Patch;
extern crate alloc;
use esp_alloc as _;
use esp_backtrace as _;

use core::include_bytes;
use little_weirdo::synth::effects::filter::FilterConfig;
use little_weirdo::synth::effects::filter::KindOfFilter;
use little_weirdo::synth::effects::overdrive::KindOfOverdrive;
use little_weirdo::synth::effects::overdrive::OverdriveConfiguration;
use little_weirdo::synth::envelope::EnvelopConfiguration;
use little_weirdo::synth::mixer::MixerConfiguration;
use little_weirdo::synth::router::RoutingConfiguration;
use little_weirdo::synth::router::VoiceToEnvelopRoute;
use little_weirdo::synth::router::VoiceToLFORoute;
use little_weirdo::synth::wavetable_oscillator::WaveTableLoFreqOscillatorConfig;
use little_weirdo::synth::wavetable_oscillator::WaveTableOscillatorConfig;
use little_weirdo::synth::patch::SynthConfiguration;
use little_weirdo::synth::patch::SynthMode;
use little_weirdo::synth::data::wavetables::Wavetables;

const SAMPLE_RATE: u32 = 44100;
const DELAY_US: u32 = 10000000 / SAMPLE_RATE;
#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // init CPU
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);


    println!("used {:?}", esp_alloc::HEAP.used());



    let mut fix = BoxedWavetables::new();
    fix.add(BoxedWavetable::new(include_bytes!("wav0.lwt")));
    fix.add(BoxedWavetable::new(include_bytes!("wav1.lwt")));


   let test3 = Rc::new(fix);


   

   let test4: Rc<BoxedWavetables> = Rc::clone(&test3);
   
   println!("Strong Count = {:?}", Rc::strong_count(&test3));
 

   let patch = Patch 
   {
    synth_config: SynthConfiguration {
        mode: SynthMode::Mono,
    },
    voices: [
        WaveTableOscillatorConfig {
            soundbank_index: 0,
            glide: false,
            glide_rate: 200,
            detune: 0,
            freq: 440,
            freq_detune: 0,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 0,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 440,
            freq_detune: 1,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 1,
            glide: false,
            glide_rate: 200,
            detune: 0,
            freq: 440,
            freq_detune: 2,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 9,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 440,
            freq_detune: 3,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 1,
            glide: true,
            glide_rate: 200,
            detune: 0,
            freq: 440,
            freq_detune: 0,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 9,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 440,
            freq_detune: 1,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 1,
            glide: false,
            glide_rate: 200,
            detune: 0,
            freq: 440,
            freq_detune: 2,
        },
        WaveTableOscillatorConfig {
            soundbank_index: 9,
            glide: false,
            glide_rate: 0,
            detune: 0,
            freq: 440,
            freq_detune: 3,
        },
    ],
    envelops: [
        EnvelopConfiguration {
            attack_time: 10,
            decay_time: 100,
            release_time: 300,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 200,
            decay_time: 10,
            release_time: 100,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 10,
            decay_time: 100,
            release_time: 300,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 200,
            decay_time: 10,
            release_time: 100,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 10,
            decay_time: 100,
            release_time: 300,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 200,
            decay_time: 10,
            release_time: 100,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 10,
            decay_time: 100,
            release_time: 300,
            sustain_level: 90,
        },
        EnvelopConfiguration {
            attack_time: 200,
            decay_time: 10,
            release_time: 100,
            sustain_level: 90,
        },
    ],
    lfos: [
        WaveTableLoFreqOscillatorConfig {
            soundbank_index: 2,
            time: 200,
        },
        WaveTableLoFreqOscillatorConfig {
            soundbank_index: 2,
            time: 200,
        },
        WaveTableLoFreqOscillatorConfig {
            soundbank_index: 2,
            time: 200,
        },
        WaveTableLoFreqOscillatorConfig {
            soundbank_index: 2,
            time: 200,
        },
    ],
    routering_config: RoutingConfiguration {
        voices_to_envelop: [
            VoiceToEnvelopRoute { env: 0 },
            VoiceToEnvelopRoute { env: 1 },
            VoiceToEnvelopRoute { env: 2 },
            VoiceToEnvelopRoute { env: 3 },
            VoiceToEnvelopRoute { env: 4 },
            VoiceToEnvelopRoute { env: 5 },
            VoiceToEnvelopRoute { env: 6 },
            VoiceToEnvelopRoute { env: 7 },
        ],
        voice_to_lfo: [
            VoiceToLFORoute {
                enable: false,
                voices: [1,255],
            },
            VoiceToLFORoute {
                enable: false,
                voices: [1,255],
            },
            VoiceToLFORoute {
                enable: false,
                voices: [1,255],
            },
            VoiceToLFORoute {
                enable: false,
                voices: [1,255],
            },
        ],
        lfo_to_filter: false,
    },
    filter_config: FilterConfig {
        cutoff_frequency: 1_000,
        resonance: 6_000,
        enabled: false,
        kind_of_filter: KindOfFilter::High,
    },
    mixer_config: MixerConfiguration {
        gain_voices: [10, 0, 0, 0, 0, 0, 0, 0],
        gain_main: 50,
    },
    overdrive_config: OverdriveConfiguration {
        threshold: 1000,
        kind: KindOfOverdrive::Softer,
        enabled: false,
    },
    };
    println!("used {:?}", esp_alloc::HEAP.used());
    


    let mut synth: synth::Synth =
        synth::Synth::new(SAMPLE_RATE as u16, &patch, Rc::clone(&test3));
  
    let mut sum = 0;
    let mut overrun = 0;
    let mut high = 0;
    let mut low: i64 = DELAY_US as i64;
    let mut moment: u32 = 0;
    for _x in 0..5 {
        let start_time = rtc.current_time().and_utc().timestamp_micros();
        synth.load_patch(&patch);
        let stop_time = rtc.current_time().and_utc().timestamp_micros();
        esp_println::println!("> patch change in µs (max 20µs) {}", stop_time - start_time);
    }
    
    for _x in 0..5 {
        synth.note_on(60, 100);
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
        esp_println::println!("----- 1 sec cycle @ 444100Hz (max 22µs ) -----");
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



