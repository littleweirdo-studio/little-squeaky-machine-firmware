extern crate alloc;

use alloc::rc::Rc;
use alloc::vec::Vec;
use driver_sgtl5000::SGTL5000;
use embedded_storage::ReadStorage;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::gpio::NoPin;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::Cpu;
use esp_hal::{
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    peripherals::Peripherals,
};
use esp_println::println;
use esp_storage::FlashStorage;
use fugit::RateExtU32;
use little_weirdo::synth::sampler::BoxedSample;
use smart_leds::colors;
use usbd_midi::message::control_function::ControlFunction;
use usbd_midi::message::Message;

use crate::{led, MIDI_MESSAGE_CHANNEL};
use little_weirdo::synth::{
    self,
    data::wavetables::{BoxedWavetable, BoxedWavetables},
    effects::{
        filter::{FilterConfig, KindOfFilter},
        overdrive::{KindOfOverdrive, OverdriveConfiguration},
    },
    envelope::EnvelopConfiguration,
    mixer::MixerConfiguration,
    patch::{Patch, SynthConfiguration, SynthMode},
    router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute},
    wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig},
};

pub fn synth_loop() {
    const SAMPLE_RATE: u32 = 44100;
    const AMOUNT_OF_SAMPLES_BUFFERING: usize = 1100;
    const BUFFER_SIZE: usize = 4 * AMOUNT_OF_SAMPLES_BUFFERING;

    esp_println::println!(
        "-:= > Starting LttL Weirdɵ on core {}",
        Cpu::current() as usize
    );

    esp_println::println!("-:= > Starting LttL Weirdɵ @ 16bits / {} Hz", SAMPLE_RATE);
    let mut storage = FlashStorage::new();

    let mut wt_on_heap = BoxedWavetables::new();
    let mut wt_read_buffer = [0u8; 1200];
    for wt in 0..10 {
        storage.read(0x00220000 + wt * 1200, &mut wt_read_buffer);
        wt_on_heap.add(BoxedWavetable::new(&wt_read_buffer));
    }
    let wt = Rc::new(wt_on_heap);

    let mut sample_read_buffer = [0u8; 1000];

    let mut sample: Vec<u8> = Vec::with_capacity(1 * 1_000);
    for wt in 0..1 {
        let offset = 0x00220000 + 10 * 1200 + wt * 1000;
        let result = storage.read(offset, &mut sample_read_buffer);
        match result {
            Ok(_) => {}
            Err(error) => esp_println::println!("Error {:?}", error),
        }
        let mut buffer = sample_read_buffer.to_vec();
        sample.append(&mut buffer);
    }

    let boxed = BoxedSample::new(sample);

    let sample_on_heap = Rc::new(boxed);

    esp_println::println!("-:= > Heap used {} KB", esp_alloc::HEAP.used() / 1000);

    let patch = Patch {
        synth_config: SynthConfiguration {
            mode: SynthMode::OctoPoly,
        },
        voices: [
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 2,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
        ],
        envelops: [
            EnvelopConfiguration {
                attack_time: 20,
                decay_time: 100,
                release_time: 1000,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 20,
                decay_time: 100,
                release_time: 1000,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 20,
                decay_time: 100,
                release_time: 1000,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 20,
                decay_time: 100,
                release_time: 1000,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 20,
                decay_time: 100,
                release_time: 1000,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 20,
                decay_time: 100,
                release_time: 1000,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 20,
                decay_time: 100,
                release_time: 1000,
                sustain_level: 50,
            },
            EnvelopConfiguration {
                attack_time: 100,
                decay_time: 100,
                release_time: 1000,
                sustain_level: 50,
            },
        ],
        lfos: [
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 0,
                time: 50,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 4,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 4,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 4,
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
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
            ],
            lfo_to_filter: false,
        },
        filter_config: FilterConfig {
            cutoff_frequency: 1_000,
            resonance: 6_000,
            enabled: true,
            kind_of_filter: KindOfFilter::Low,
        },
        mixer_config: MixerConfiguration {
            gain_voices: [10, 10, 10, 10, 10, 10, 10, 10],
            gain_main: 20,
        },
        overdrive_config: OverdriveConfiguration {
            threshold: 1000,
            kind: KindOfOverdrive::Softer,
            enabled: false,
        },
    };

    let mut synth: synth::Synth = synth::Synth::new(
        SAMPLE_RATE as u16,
        &patch,
        Rc::clone(&wt),
        Rc::clone(&sample_on_heap),
    );

    let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, BUFFER_SIZE);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma_channel = peripherals.DMA_CH0;

        let mut i2s = I2s::new(
            peripherals.I2S0,
            Standard::Philips,
            DataFormat::Data16Channel16,
            SAMPLE_RATE.Hz(),
            dma_channel,
            rx_descriptors,
            tx_descriptors,
        );

        i2s = i2s.with_mclk(peripherals.GPIO38);
        let mut i2s_tx = i2s
            .i2s_tx
            .with_bclk(peripherals.GPIO40)
            .with_ws(peripherals.GPIO39)
            .with_dout(peripherals.GPIO42)
            .build();


        esp_println::println!(
            "-:= > Starting LttL Weirdɵ DMA transfer with a {} ms buffer",
            (AMOUNT_OF_SAMPLES_BUFFERING * 1000 / SAMPLE_RATE as usize * 1000) / 1000
        );

        let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            match MIDI_MESSAGE_CHANNEL.try_receive() {
                Ok(message) => match message {
                    Message::NoteOn(_channel, note, velocity) => {
                        synth.note_on(note.into(), velocity.into())
                    }
                    Message::NoteOff(_channel, note, _velocity) => {
                        synth.note_off(note.into())
                    }
                    Message::ProgramChange(_channel, program) => {
                        let program_change: u8 = program.into();
                        match program_change {
                            0x00 => {
                                //synth.load_patch(&SOUND_BANK_PURE_ELEKTRO.patches[0]);
                            }
                            0x01 => {
                                //synth.load_patch(&SOUND_BANK_PURE_ELEKTRO.patches[1]);
                            }
                            0x02 => {
                                //synth.load_patch(&SOUND_BANK_PURE_ELEKTRO.patches[2]);
                            }
                            0x03 => {
                                //synth.load_patch(&SOUND_BANK_PURE_ELEKTRO.patches[3]);
                            }
                            _ => {}
                        }
                    }
                    Message::ControlChange(_channel, control_change, value) => {
                        let new_value: u8 = value.into();
                        if control_change == ControlFunction::CHANNEL_VOLUME_7 {
                            synth.change_main_volume(new_value);
                        }
                        if control_change == ControlFunction::SOUND_CONTROLLER_1_70 {
                           synth.change_cutoff(new_value);
                        }
                        if control_change == ControlFunction::SOUND_CONTROLLER_2_71 {
                            synth.change_resonance(new_value);
                        }
                    }
                    _ => {}
                },
                _error => {}
            };

            match transfer.available() {
                Ok(num_bytes) => {
                    let avail = usize::min(BUFFER_SIZE, num_bytes);
                    for i in 0..(avail / 4) {
                        let output = synth.clock_and_output();
                        let left = output[0].to_be_bytes();
                        let right = output[1].to_be_bytes();
                        // left
                        buffer[i * 4] = left[1];
                        buffer[i * 4 + 1] = left[0];
                        //right
                        buffer[i * 4 + 2] = right[1];
                        buffer[i * 4 + 3] = right[0];
                    }
                    let _ = transfer.push(&buffer[0..avail]);
                }
                Err(error) => {
                    println!("Error: {:?}", error);
                }
            }
        }
    }
}
