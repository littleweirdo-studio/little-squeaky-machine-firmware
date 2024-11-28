use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel as Chn};
use hal::i2s::{DataFormat, I2s, Standard};
use little_weirdo::synth;
use little_weirdo::synth::envelope::Envelop;
use little_weirdo::synth::filter::FilterConfig;
use little_weirdo::synth::oscillator::Waveform;
use little_weirdo::synth::patch::Patch;

use crate::midi;
use esp32_hal as hal;
use esp_hal_common::i2s::I2sWriteDma;
use hal::clock::ClockControl;
use hal::Rng;
use hal::{
    dma::DmaPriority, pdma::Dma, peripherals::Peripherals, prelude::*, timer::TimerGroup, IO,
};
use midi::MidiMessage;

#[embassy_executor::task]
pub async fn synth_task(
    midi_messages_channel: &'static Chn<CriticalSectionRawMutex, MidiMessage, 10>,
) {
    let mut tx_descriptors = [0u32; 20 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];
    let mut transfer;
    let buffer: &mut [u8; 22050] = dma_buffer();

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let system = peripherals.SYSTEM.split();
        let clocks = ClockControl::max(system.clock_control).freeze();

        let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

        let dma = Dma::new(system.dma);
        let dma_channel = dma.i2s0channel;

        let i2s = I2s::new(
            peripherals.I2S0,
            Standard::Philips,
            DataFormat::Data16Channel16,
            22500u32.Hz(),
            dma_channel.configure(
                false,
                &mut tx_descriptors,
                &mut rx_descriptors,
                DmaPriority::Priority0,
            ),
            &clocks,
        );

        let i2s_tx = i2s
            .i2s_tx
            .with_bclk(io.pins.gpio27)
            .with_ws(io.pins.gpio25)
            .with_dout(io.pins.gpio26)
            .build();
        transfer = i2s_tx.write_dma_circular(buffer).unwrap();
    }
    let patch = Patch {
        voice_1: Waveform::SawTooth,
        voice_1_mix_level: 80,
        voice_2: Waveform::Square25,
        voice_2_mix_level: 80,
        lfo_1: 17,
        lfo_1_mix_level: 0,
        voice_1_env: Envelop {
            attack_time: 20,
            decay_time: 100,
            release_time: 400,
            sustain_level: 80,
        },
        voice_2_env: Envelop {
            attack_time: 20,
            decay_time: 100,
            release_time: 400,
            sustain_level: 50,
        },

        voice_1_detune: 0,
        voice_2_detune: 0,
        filter_config: FilterConfig {
            cutoff_frequency: 368,
            filter_on: true,
            _24db: false,
        },
        main_gain: 50,
        glide: true,
        glide_rate: 5,
        mono: true,
    };

    let mut synth: synth::Synth = synth::Synth::new(22500, patch);
    let mut filler = [0u8; 50];
    loop {
        let avail = transfer.available();
        if avail > 0 {
            let avail = usize::min(50, avail);
            for i in 0..(avail / 4) {
                let output: i16 = synth.clock_and_output();
                match midi_messages_channel.try_receive() {
                    Ok(message) => match message {
                        MidiMessage::NoteOn(channel, note) => {
                            synth.note_on(0x00, note.key, note.value)
                        }
                        MidiMessage::NoteOff(channel, note) => synth.note_off(0x00, note.key),
                        _ => {}
                    },
                    Err(_) => {}
                };
                let bytes = output.to_be_bytes();
                filler[i * 4] = bytes[1];
                filler[i * 4 + 1] = bytes[0];
                filler[i * 4 + 2] = bytes[1];
                filler[i * 4 + 3] = bytes[0];
            }
            transfer.push(&filler[0..avail]).unwrap();
        };
    }
}

fn dma_buffer() -> &'static mut [u8; 22050] {
    static mut BUFFER: [u8; 22050] = [0u8; 22050];
    unsafe { &mut BUFFER }
}
