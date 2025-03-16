use esp_backtrace as _;
//use embassy_time::{Duration, Timer};
use esp_hal::{gpio::Input, peripherals::Peripherals};
use esp_println::println;

use crate::{MIDI_MESSAGE_CHANNEL, MIDI_OUT_MESSAGE_CHANNEL};
extern crate alloc;

#[embassy_executor::task]
pub async fn touch_input_task() {
    let mut pin;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        pin = Input::new(peripherals.GPIO0, esp_hal::gpio::Pull::Up);
    }
    loop {
        pin.wait_for_low().await;
        println!("test");
        MIDI_MESSAGE_CHANNEL
            .send(midi_msg::MidiMsg::ChannelVoice {
                channel: midi_msg::Channel::Ch1,
                msg: midi_msg::ChannelVoiceMsg::NoteOn {
                    note: 60,
                    velocity: 100,
                },
            })
            .await;

        MIDI_OUT_MESSAGE_CHANNEL
            .send(midi_msg::MidiMsg::ChannelVoice {
                channel: midi_msg::Channel::Ch1,
                msg: midi_msg::ChannelVoiceMsg::NoteOn {
                    note: 60,
                    velocity: 100,
                },
            })
            .await;

        pin.wait_for_high().await;
        MIDI_MESSAGE_CHANNEL
            .send(midi_msg::MidiMsg::ChannelVoice {
                channel: midi_msg::Channel::Ch1,
                msg: midi_msg::ChannelVoiceMsg::NoteOff {
                    note: 60,
                    velocity: 100,
                },
            })
            .await;

        MIDI_OUT_MESSAGE_CHANNEL
            .send(midi_msg::MidiMsg::ChannelVoice {
                channel: midi_msg::Channel::Ch1,
                msg: midi_msg::ChannelVoiceMsg::NoteOff {
                    note: 60,
                    velocity: 100,
                },
            })
            .await;
    }
}
