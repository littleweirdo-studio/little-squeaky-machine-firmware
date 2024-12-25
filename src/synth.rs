

extern crate alloc;
use esp_println::println;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaPriority},
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    peripherals::Peripherals,
    prelude::*,
};
use esp_hal::Cpu;
use little_weirdo_soundbanks::{patches::Patches, soundbanks::SOUND_BANK_PURE_ELEKTRO};
use usbd_midi::data::midi::message::control_function::ControlFunction;
use usbd_midi::data::midi::message::Message;

use little_weirdo::synth::{self};
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

    let mut synth: synth::Synth =
        synth::Synth::new(SAMPLE_RATE as u16, Patches::get_patch(Patches::BassGuitar), &SOUND_BANK_PURE_ELEKTRO);
    let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, BUFFER_SIZE);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.channel0;

        let i2s = I2s::new(
            peripherals.I2S0,
            Standard::Philips,
            DataFormat::Data16Channel16,
            SAMPLE_RATE.Hz(),
            dma_channel.configure(false, DmaPriority::Priority0),
            rx_descriptors,
            tx_descriptors,
        );

        let mut i2s_tx = i2s
            .i2s_tx
            .with_bclk(peripherals.GPIO6)
            .with_ws(peripherals.GPIO4)
            .with_dout(peripherals.GPIO5)
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
                    Message::NoteOff(_channel, note, _velocity) => synth.note_off(note.into()),
                    Message::ProgramChange(_channel, program) => {
                        let program_change: u8 = program.into();
                        match program_change {
                            0x00 => {
                                synth.load_patch(Patches::get_patch(Patches::BassGuitar));
                            }
                            0x01 => {
                                synth.load_patch(Patches::get_patch(Patches::BassGuitarFourth));
                            }
                            0x02 => {
                                synth.load_patch(Patches::get_patch(Patches::BassGuitarMajor));
                            }
                            0x03 => {
                                synth.load_patch(Patches::get_patch(Patches::WeirdScience));
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
