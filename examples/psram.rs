#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_alloc as _;
use esp_hal::peripherals::PSRAM;
use esp_hal::psram::{self, PsramSize};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::{
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};
use esp_println::println;
use little_weirdo::synth::data::wavetables::Wavetable;
use little_weirdo::synth::effects::filter::{FilterConfig, KindOfFilter};
use little_weirdo::synth::effects::overdrive::{KindOfOverdrive, OverdriveConfiguration};
use little_weirdo::synth::envelope::EnvelopConfiguration;
use little_weirdo::synth::mixer::MixerConfiguration;
use little_weirdo::synth::patch::Patch;
use little_weirdo::synth::router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute};
use little_weirdo::synth::wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig, Waveform};
use little_weirdo::synth::{self, patches::Patches};

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
    println!("> test");
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);
    println!("> used :: {:?} KB", esp_alloc::HEAP.used()/1000); 
    println!("> used :: {:?} KB", esp_alloc::HEAP.free()/1000); 
  
    let patch:Box<Patch> =  Box::new(Patch { 
        voices: [
            WaveTableOscillatorConfig {
                waveform: Waveform::Bass,
                glide: true,
                glide_rate: 10,
                detune: 0,
                freq: 440,
            }, 
            WaveTableOscillatorConfig {
                waveform: Waveform::Bass,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
            }, 
            WaveTableOscillatorConfig {
                waveform: Waveform::Bass,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
            }, 
            WaveTableOscillatorConfig {
                waveform: Waveform::Bass,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
            }, 
        ],
        envelops: [
            EnvelopConfiguration { 
                attack_time: 100, 
                decay_time: 10, 
                release_time: 100, 
                sustain_level: 80 
            }, 
            EnvelopConfiguration { 
                attack_time: 100, 
                decay_time: 10, 
                release_time: 100, 
                sustain_level: 80 
            }, 
            EnvelopConfiguration { 
                attack_time: 100, 
                decay_time: 10, 
                release_time: 100, 
                sustain_level: 80 
            }, 
            EnvelopConfiguration { 
                attack_time: 100, 
                decay_time: 10, 
                release_time: 100, 
                sustain_level: 80 
            }, 
        ],
        lfo: WaveTableLoFreqOscillatorConfig {
            waveform: Waveform::Sine,
            time: 50,
        },
        routering_config: RoutingConfiguration {
            voices_to_envelop: [
                VoiceToEnvelopRoute {
                    env: 0,
                },
                VoiceToEnvelopRoute {
                    env: 1,
                },  
                VoiceToEnvelopRoute {
                    env: 2,
                },  
                VoiceToEnvelopRoute {
                    env: 3,
                },  
                ],
            voice_to_lfo : VoiceToLFORoute {
                enable: false,
                voice: 0,
            },
            lfo_to_filter: true,
        },   
        filter_config: FilterConfig { 
            cutoff_frequency: 1_000, 
            resonance: 5_000,
            enabled: true, 
            kind_of_filter: KindOfFilter::Low,
        }, 
        mixer_config: MixerConfiguration { 
            gain_voices: [
                60,
                0,
                0,
                0,
            ],
            gain_main: 50, 
        },
        overdrive_config: OverdriveConfiguration { 
            threshold: 2000, 
            kind: KindOfOverdrive::Softer, 
            enabled: false, 
        }
    }
    );
    println!("> used :: {:?} KB", esp_alloc::HEAP.used()); 
    println!("> used :: {:?} KB", esp_alloc::HEAP.free()); 
  
    let mut synth:Box<synth::Synth> = Box::new(synth::Synth::new(44100, *patch));

  
    println!("> used :: {:?} KB", esp_alloc::HEAP.used()); 
    println!("> used :: {:?} KB", esp_alloc::HEAP.free()); 
  
    let v = vec![Wavetable{ data: [0;600]};10];

}




