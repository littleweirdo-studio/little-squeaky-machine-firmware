extern crate alloc;

use alloc::{string::ToString, sync::Arc, vec};
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
use little_weirdo::{effects::{bitcrunch::BitcrunchConfiguration, delay::DelayConfiguration, overdrive::{KindOfOverdrive, OverdriveConfiguration}}, synth::{self, Synth}};

use little_weirdo::{
    sampler::{
        self,
        audio_sampler::{BoxedSample, BoxedSamples},
    },
    synth::data::wavetables::{BoxedWavetable, BoxedWavetables},
};
use little_weirdo_filesystem::WeirdoFileSystem;
use midi_msg::Channel;

use crate::MIDI_MESSAGE_CHANNEL;

pub fn synth_loop() {
    const SAMPLE_RATE: u32 = 44100;
    const AMOUNT_OF_SAMPLES_BUFFERING: usize = 110;
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

    let mut wt_on_heap = BoxedWavetables::new();
    let mut wt_read_buffer = vec![0u8; 1200];
    for wt in 0..10 {
        println!("ket = {:?}", wt);
        let _ = filesystem
            .read_key_value(700 + wt, &mut wt_read_buffer)
            .unwrap();
        wt_on_heap.add(BoxedWavetable::new(&wt_read_buffer));
    }
    let wt = Arc::new(wt_on_heap);

    let mut samples_on_heap = BoxedSamples::new();
    let mut sample_read_buffer = vec![0u8; 48_000];
    for wt in 0..5 {
        let length_of_sample = filesystem
            .read_key_value(900 + wt, &mut sample_read_buffer)
            .unwrap();
        samples_on_heap.add(BoxedSample::new(
            &sample_read_buffer[..length_of_sample as usize],
        ));
    }

    let sp = Arc::new(samples_on_heap);

    esp_println::println!("-:= > \n{}-:= >", esp_alloc::HEAP.stats());

    let mut patch_bytes = [0u8; 256];

    let _ = filesystem.read_key_value(102, &mut patch_bytes).unwrap();

    let patch: little_weirdo::synth::patch::Patch = postcard::from_bytes(&patch_bytes).unwrap();

    let mut synth: synth::Synth = Synth::new(SAMPLE_RATE as u16, &patch, Arc::clone(&wt));

    let sampler_patch = little_weirdo::sampler::patch::Patch {
        name: "Sampler patch".to_string(),
        overdrive_config: OverdriveConfiguration { threshold: 100, kind: KindOfOverdrive::Hard , enabled: false },
        bitcrunch_config: BitcrunchConfiguration { enabled: false},
        delay_config: DelayConfiguration {enabled: false, delay_time: 100, mix_percentage: 50, feedback: true, feedback_percentage: 50 },
    };

    let mut sampler: sampler::Sampler = sampler::Sampler::new(SAMPLE_RATE as u16, &sampler_patch, Arc::clone(&sp));

    let (_, _rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, BUFFER_SIZE);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma_channel = peripherals.DMA_CH0;

        let mut i2s = I2s::new(
            peripherals.I2S0,
            dma_channel,
            Config::new_tdm_philips()
                .with_sample_rate(Rate::from_hz(44100))
                .with_data_format(DataFormat::Data16Channel16)
                .with_channels(Channels::STEREO),
        )
        .unwrap();

        i2s = i2s.with_mclk(peripherals.GPIO38);

        let mut i2s_tx = i2s
            .i2s_tx
            .with_bclk(peripherals.GPIO40)
            .with_ws(peripherals.GPIO39)
            .with_dout(peripherals.GPIO42)
            .build(tx_descriptors);

        esp_println::println!(
            "-:= > Starting LttL Weirdɵ DMA transfer with a {} ms buffer",
            (AMOUNT_OF_SAMPLES_BUFFERING * 1000 / SAMPLE_RATE as usize * 1000) / 1000
        );

        let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];

        loop {
            match MIDI_MESSAGE_CHANNEL.try_receive() {
                Ok(message) => match message {
                    midi_msg::MidiMsg::ChannelVoice { channel, msg } => {
                        match channel {
                            Channel::Ch1 => {
                                match msg {
                                    midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
                                        //           println!("note on");
                                        synth.note_on(note, velocity);
                                    }
                                    midi_msg::ChannelVoiceMsg::NoteOff { note, velocity: _ } => {
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

                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Channel::Ch2 => match msg {
                                midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
                                    sampler.note_on(note, velocity);
                                }
                                midi_msg::ChannelVoiceMsg::NoteOff { note, velocity: _ } => {
                                    sampler.note_off(note);
                                }
                                _ => {}
                            },
                            _ => {}
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
                        let output_sampler = sampler.clock_and_output();
                        let output_synth = synth.clock_and_output();

                        let mix = output_sampler[0] + output_synth[0];

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
