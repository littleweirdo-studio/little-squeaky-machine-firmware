extern crate alloc;



use alloc::vec::Vec;
use alloc::{boxed::Box, rc::Rc};
use driver_sgtl5000::SGTL5000;
use embedded_storage::Storage;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::Cpu;
use esp_hal::{
    dma::{Dma, DmaPriority},
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    peripherals::Peripherals,
    prelude::*,
};
use esp_println::println;
use little_weirdo::synth::data::wavetables::Wavetables;
use little_weirdo::synth::sampler::BoxedSample;
use usbd_midi::data::midi::message::control_function::ControlFunction;
use usbd_midi::data::midi::message::Message;
use esp_storage::FlashStorage;
use embedded_storage::{ReadStorage};

use crate::MIDI_MESSAGE_CHANNEL;
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


    let mut sample:Vec<u8> = Vec::with_capacity(328 * 1_000);
    for wt in 0..328 {
        let offset = 0x00220000 + 10 * 1200 + wt * 1000;
        let result = storage.read(offset, &mut sample_read_buffer);
        match result {
            Ok(_) => {},
            Err(error) => esp_println::println!("Error {:?}", error),
        }
        let mut buffer = sample_read_buffer.to_vec();
        sample.append(&mut buffer);
    }

    let boxed = BoxedSample::new(sample);
    

    let  sample_on_heap = Rc::new(boxed);

    esp_println::println!("-:= > Heap used {} KB", esp_alloc::HEAP.used()/1000);
  
    let patch = Patch {
        synth_config: SynthConfiguration {
            mode: SynthMode::Mono,
        },
        voices: [
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
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
                soundbank_index: 1,
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
                freq_detune: 4,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
                freq_detune: 5,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 6,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
                freq_detune: 7,
            },
        ],
        envelops: [
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 100,
                release_time: 100,
                sustain_level: 80,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
        ],
        lfos: [
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 0,
                time: 50,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 1,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 1,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 1,
                time: 200,
            },
        ],
        routering_config: RoutingConfiguration {
            voices_to_envelop: [
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
            ],
            voice_to_lfo: [
                VoiceToLFORoute {
                    enable: true,
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
            enabled: false,
            kind_of_filter: KindOfFilter::Low,
        },
        mixer_config: MixerConfiguration {
            gain_voices: [20, 0, 0, 0, 0, 0, 0, 0],
            gain_main: 20,
        },
        overdrive_config: OverdriveConfiguration {
            threshold: 1000,
            kind: KindOfOverdrive::Softer,
            enabled: false,
        },
    };
  
    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE as u16, &patch, Rc::clone(&wt), Rc::clone(&sample_on_heap));
    
    let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, BUFFER_SIZE);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.channel0;

        let mut i2s = I2s::new(
            peripherals.I2S0,
            Standard::Philips,
            DataFormat::Data16Channel16,
            SAMPLE_RATE.Hz(),
            dma_channel.configure(true, DmaPriority::Priority9),
            rx_descriptors,
            tx_descriptors,
        );

       
   
        i2s = i2s.with_mclk(peripherals.GPIO4);
        let mut i2s_tx = i2s
            .i2s_tx
            .with_bclk(peripherals.GPIO5)
            .with_ws(peripherals.GPIO6)
            .with_dout(peripherals.GPIO7)
            .build();

            let mut config = Config::default();
            config.frequency =  400u32.kHz();

            let mut config = Config::default();
            let mut i2c = I2c::new(peripherals.I2C0, config)
                .with_sda(peripherals.GPIO41)
                .with_scl(peripherals.GPIO42);
        
        

            let mut sgtl500 = SGTL5000::new(i2c);
    
    println!("Check if the SGTL5000 is ready and respond to commands\n");
    println!(
        "--> SGTL5000 :: Chip Ready {:?}",
        sgtl500.ready().unwrap()
    );

    println!("Initialize the SGTL5000\n");
    println!(
        "--> SGTL5000 :: Chip Revision {:#x}",
        sgtl500.get_chip_revision().unwrap()
    );
    println!("--> SGTL5000 :: Codec setup {:?}", sgtl500.init());


    println!(
        "--> SGTL5000 ::Volume to 40 {:?}",
        sgtl500.headphone_volume(70)
    );


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
                    Message::NoteOff(_channel, note, _velocity) => synth.note_off(note.into()),
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
