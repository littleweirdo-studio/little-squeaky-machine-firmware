extern crate alloc;

use alloc::{string::String, sync::Arc, vec};

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    dma_circular_buffers,
    i2s::master::{Channels, Config, DataFormat, I2s},
    peripherals::Peripherals,
    system::Cpu,
    time::Rate,
};
use esp_println::println;
use esp_storage::FlashStorage;
use little_weirdo::{
    synth::data::waveforms::{BoxedWaveform, BoxedWaveforms},
    synth::{
        self, Synth,
        data::patches::{BoxedPatch, BoxedPatches},
        patch::Patch,
    },
    wavetable::data::patches::BoxedSamplerPatches,
    wavetable::sample_voice::{BoxedSample, BoxedSamples},
};

use little_weirdo_filesystem::WeirdoFileSystem;
use midi_msg::Channel;

use crate::MIDI_MESSAGE_CHANNEL;

use serde::Deserialize;
use serde::Serialize;
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SoundBank {
    pub name: String,
    pub amount_of_patches: u16,
    pub amount_of_waveforms: u16,
    pub amount_of_samples: u16,
    pub amound_of_sampler_patches: u16,
}

pub fn synth_loop() {
    const SAMPLE_RATE: u32 = 44100;
    const AMOUNT_OF_SAMPLES_BUFFERING: usize = 1100;
    const BUFFER_SIZE: usize = 4 * AMOUNT_OF_SAMPLES_BUFFERING;

    esp_println::println!(
        "-:= > Starting Little Weirdo on core {}",
        Cpu::current() as usize
    );

    esp_println::println!("-:= > Starting Little Weirdo @ 16bits / {} Hz", SAMPLE_RATE);
    let storage;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        storage = FlashStorage::new(peripherals.FLASH);
    }

    let mut filesystem: WeirdoFileSystem<FlashStorage> =
        WeirdoFileSystem::new(storage, 0x00220000, 0x100000);

    //get settings

    let mut buf = vec![0u8; 256];

    let _ = filesystem.read_key_value(000, buf.as_mut_slice());
    let sound_bank: SoundBank = postcard::from_bytes(&buf).unwrap();

    println!("Soundbank Loaded = {}", sound_bank.name);

    let mut wt_on_heap = BoxedWaveforms::new();
    let mut wt_read_buffer = vec![0u8; 1200];
    for wt in 0..sound_bank.amount_of_waveforms as u32 {
        //        println!("key = {:?}", wt);
        let _ = filesystem
            .read_key_value(700 + wt, wt_read_buffer.as_mut_slice())
            .unwrap();
        wt_on_heap.add(BoxedWaveform::new(&wt_read_buffer));
    }
    let wt = Arc::new(wt_on_heap);

    let mut samples_on_heap = BoxedSamples::new();

    for wt in 0..sound_bank.amount_of_samples as u32 {
        //   println!("Load Sample from flash and create it with ID {}", wt);
        let size_of_value = filesystem.size_of_key_value(900 + wt).unwrap();
        let mut sample: vec::Vec<u8> = vec![0u8; size_of_value as usize]; //vec::Vec::with_capacity(size_of_value as usize);

        let _length_of_sample = filesystem
            .read_key_value(900 + wt, sample.as_mut_slice())
            .unwrap();
        samples_on_heap.add(BoxedSample::new(sample));
    }

    let _sp = Arc::new(samples_on_heap);

    let mut patches_on_heap = BoxedPatches::new();
    let mut patch_read_buffer = vec![0u8; 512];
    for pt in 0..sound_bank.amount_of_patches as u32 {
        let length_of_patch = filesystem
            .read_key_value(100 + pt, &mut patch_read_buffer)
            .unwrap();
        let patch: Patch =
            postcard::from_bytes(&patch_read_buffer[..length_of_patch as usize]).unwrap();
        println!("patch {}", patch.name);
        patches_on_heap.add(BoxedPatch::new(patch));
    }

    let patches = Arc::new(patches_on_heap);

    //

    let sampler_patches_on_heap = BoxedSamplerPatches::new();
    let mut sampler_patch_read_buffer = vec![0u8; 50];
    for pt in 0..sound_bank.amound_of_sampler_patches as u32 {
        let _length_of_patch = filesystem
            .read_key_value(200 + pt, &mut sampler_patch_read_buffer)
            .unwrap();
        // let patch: sampler::patch::Patch =
        //     postcard::from_bytes(&sampler_patch_read_buffer[..length_of_patch as usize]).unwrap();
        // println!("sampler patch {}", patch.name);
        // sampler_patches_on_heap.add(BoxedSamplerPatch::new(patch));
    }

    let _sampler_patches = Arc::new(sampler_patches_on_heap);

    esp_println::println!("-:= > \n{}-:= >", esp_alloc::HEAP.stats());

    let mut synth: synth::Synth =
        Synth::new(SAMPLE_RATE as u16, 5, Arc::clone(&patches), Arc::clone(&wt));

    // let mut sampler: sampler::Sampler = sampler::Sampler::new(
    //     SAMPLE_RATE as u16,
    //     0,
    //     Arc::clone(&sampler_patches),
    //     Arc::clone(&sp),
    // );

    //let mut recorder = AudioRecorder::new(SAMPLE_RATE, SAMPLE_RATE as usize * 2* 10);

    let (_rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) =
        dma_circular_buffers!(BUFFER_SIZE, BUFFER_SIZE);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma_channel = peripherals.DMA_CH0;

        let mut i2s = I2s::new(
            peripherals.I2S0,
            dma_channel,
            Config::new_tdm_philips()
                .with_signal_loopback(false)
                .with_sample_rate(Rate::from_hz(44100))
                .with_data_format(DataFormat::Data16Channel16)
                .with_channels(Channels::STEREO),
        )
        .unwrap();

        i2s = i2s.with_mclk(peripherals.GPIO42);

        let mut i2s_tx = i2s
            .i2s_tx
            .with_bclk(peripherals.GPIO40)
            .with_ws(peripherals.GPIO41)
            .with_dout(peripherals.GPIO38)
            .build(tx_descriptors);

        let _i2s_rx = i2s
            .i2s_rx
            .with_din(peripherals.GPIO39)
            .build(rx_descriptors);

        esp_println::println!(
            "-:= > Starting LttL Weirdɵ DMA transfer with a {} ms buffer",
            (AMOUNT_OF_SAMPLES_BUFFERING * 1000 / SAMPLE_RATE as usize * 1000) / 1000
        );

        let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
        //  let mut receiver = i2s_rx.read_dma_circular(rx_buffer).unwrap();

        let mut buffer = [0u8; BUFFER_SIZE];
        // let mut buffer_receiver =  [0u8; BUFFER_SIZE];

        let mut midi_message_receiver = MIDI_MESSAGE_CHANNEL.subscriber().unwrap();

        loop {
            //TODO fix this hack
            if midi_message_receiver.available() > 0 {
                match midi_message_receiver.try_next_message_pure() {
                    Some(msg) => {
                        match msg {
                            midi_msg::MidiMsg::ChannelVoice { channel, msg } => {
                                match channel {
                                    Channel::Ch2 => {
                                        match msg {
                                            midi_msg::ChannelVoiceMsg::ProgramChange {
                                                program,
                                            } => {
                                                synth.load_patch(program);
                                            }

                                            midi_msg::ChannelVoiceMsg::NoteOn {
                                                note,
                                                velocity,
                                            } => {
                                                  println!("note on");
                                                synth.note_on(note, velocity);
                                            }
                                            midi_msg::ChannelVoiceMsg::NoteOff {
                                                note,
                                                velocity: _,
                                            } => {
                                                synth.note_off(note);
                                            }
                                            _ => {}
                                        }
                                    }
                                    Channel::Ch1 => match msg {
                                        midi_msg::ChannelVoiceMsg::ProgramChange { program } => {
                                            synth.load_patch(program);
                                        }
                                        midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
                                            //println!("note on");
                                            synth.note_on(note, velocity);
                                        }
                                        midi_msg::ChannelVoiceMsg::NoteOff {
                                            note,
                                            velocity: _,
                                        } => {
                                            synth.note_off(note);
                                        }
                                        midi_msg::ChannelVoiceMsg::ControlChange { control } => {
                                            match control {
                                                midi_msg::ControlChange::Volume(volume) => {
                                                    synth.change_main_volume(volume as u8);
                                                }

                                                midi_msg::ControlChange::SoundControl1(value) => {
                                                    synth.change_cutoff(value);
                                                }

                                                 midi_msg::ControlChange::SoundControl2(value) => {
                                                    synth.change_all_voices(value);
                                                }

                                                midi_msg::ControlChange::AttackTime(value) => {
                                                    let attack_time = if value == 0 {
                                                        5
                                                    } else {
                                                        value as i16 * 5
                                                    };
                                                    synth.change_all_envelopes(
                                                        Some(attack_time),
                                                        None,
                                                        None,
                                                        None,
                                                    );
                                                }

                                                midi_msg::ControlChange::DecayTime(value) => {
                                                    let decay_time = if value == 0 {
                                                        5
                                                    } else {
                                                        value as i16 * 5
                                                    };
                                                    synth.change_all_envelopes(
                                                        None,
                                                        Some(decay_time),
                                                        None,
                                                        None,
                                                    );
                                                }

                                                midi_msg::ControlChange::ReleaseTime(value) => {
                                                    let release_time = if value == 0 {
                                                        5
                                                    } else {
                                                        value as i16 * 5
                                                    };

                                                    synth.change_all_envelopes(
                                                        None,
                                                        None,
                                                        None,
                                                        Some(release_time),
                                                    );
                                                }

                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    },
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }
                    None => {}
                }
                if midi_message_receiver.available() == 0 {
                    midi_message_receiver.clear();
                }
            }
            //   let mut reload = false;
            /*
            match receiver.available() {
                Ok(num_bytes) => {
                    receiver.pop(&mut buffer_receiver).unwrap();
                  //  recorder.process_audio(&buffer_receiver[..num_bytes]);
                }
                Err(error) => {
                   // println!("Error: {:?}", error);
                }
            }
            */
            match transfer.available() {
                Ok(num_bytes) => {
                    let avail = usize::min(BUFFER_SIZE, num_bytes);
                    for i in 0..(avail / 4) {
                        let output_synth = synth.clock_and_output();

                        let mix = output_synth[0];

                        let left = mix.to_be_bytes();
                        let right = left;
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
