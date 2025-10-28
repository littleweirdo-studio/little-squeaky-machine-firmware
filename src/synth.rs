extern crate alloc;

use alloc::sync::Arc;
use embedded_storage::ReadStorage;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    dma_circular_buffers,
    i2s::master::{Channels, Config, DataFormat, I2s},
    peripherals::Peripherals,
    system::Cpu, time::Rate,
};
use esp_println::println;
use esp_storage::FlashStorage;
use little_weirdo::synth::data::wavetables::{BoxedWavetable, BoxedWavetables};
use little_weirdo::synth::{self, patch::Patch};

use crate::MIDI_MESSAGE_CHANNEL;

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
        let _ = storage.read(0x00220000 + wt * 1200, &mut wt_read_buffer);
        wt_on_heap.add(BoxedWavetable::new(&wt_read_buffer));
    }
    let wt = Arc::new(wt_on_heap);

    esp_println::println!("-:= > Heap used {} KB", esp_alloc::HEAP.used() / 1000);

    let patch_bytes: &[u8] = include_bytes!("patches/piano.lwp");
    let patch: Patch = postcard::from_bytes(patch_bytes).unwrap();

    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE as u16, &patch, Arc::clone(&wt));

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
        ).unwrap();

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
                    midi_msg::MidiMsg::ChannelVoice { channel: _, msg } => {
                       //      println!("{:?}", msg);
                        match msg {
                            midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
                                //           println!("note on");
                                synth.note_on(note, velocity);
                            }
                            midi_msg::ChannelVoiceMsg::NoteOff { note, velocity: _ } => {
                                synth.note_off(note);
                            }
                            midi_msg::ChannelVoiceMsg::ControlChange { control } => match control {
                                midi_msg::ControlChange::Volume(volume) => {
                                    synth.change_main_volume(volume as u8);
                                }

                                midi_msg::ControlChange::SoundControl1(value) => {
                                    synth.change_cutoff(value);
                                }

                                _ => todo!(),
                            },
                            midi_msg::ChannelVoiceMsg::HighResNoteOn {
                                note: _,
                                velocity: _,
                            } => {}
                            midi_msg::ChannelVoiceMsg::HighResNoteOff {
                                note: _,
                                velocity: _,
                            } => {}
                            midi_msg::ChannelVoiceMsg::PolyPressure {
                                note: _,
                                pressure: _,
                            } => {}
                            midi_msg::ChannelVoiceMsg::ChannelPressure { pressure: _ } => {}
                            midi_msg::ChannelVoiceMsg::ProgramChange { program: _ } => {}
                            midi_msg::ChannelVoiceMsg::PitchBend { bend: _ } => {}
                        }
                    }
                    midi_msg::MidiMsg::RunningChannelVoice { channel: _, msg: _ } => {}
                    midi_msg::MidiMsg::ChannelMode { channel: _, msg: _ } => {}
                    midi_msg::MidiMsg::RunningChannelMode { channel: _, msg: _ } => {}
                    midi_msg::MidiMsg::SystemCommon { msg: _ } => {}
                    midi_msg::MidiMsg::SystemRealTime { msg: _ } => {}
                    midi_msg::MidiMsg::SystemExclusive { msg: _ } => {}
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
