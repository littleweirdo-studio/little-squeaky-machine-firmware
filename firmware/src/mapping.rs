use esp_println::println;
use little_weirdo_gui::{EventType, MAPPING_SIGNAL, NAVIGATION_SIGNAL};
use midi_msg::Channel;
use smart_leds::{RGB8, colors};

use crate::{
    MIDI_MESSAGE_CHANNEL,
    led::{LED_MESSAGE_CHANNEL, LedMessage},
};


const COLOR:RGB8 = colors::LIGHT_PINK;
const REMAP_MIDI: [u8; 25] = [
    255, 2, 4, 255, 7, 
    1, 3, 5, 6, 8, 
    9, 11, 255, 14, 16, 
    10, 12, 13, 15, 255, 
    255, 255, 255, 255, 255,
];

const VEL: u8 = 100;
const BRIGTHNESS: u8 = 255;
#[embassy_executor::task]
pub async fn mapping_task() {
    let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();
    //C4
    let mut base = 60;

    loop {
        let event = MAPPING_SIGNAL.receive().await;
        match event.event {
            EventType::KeypadDown => {
                let remap = REMAP_MIDI[event.id as usize];
                if remap != 255 {
                    let message = midi_msg::MidiMsg::ChannelVoice {
                        channel: Channel::Ch1,
                        msg: midi_msg::ChannelVoiceMsg::NoteOn {
                            note: base + remap,
                            velocity: VEL,
                        },
                    };

                    sender.publish(message).await;
                } else {
                    println!("event {:?}", event.id);
                    match event.id {
                        //C2
                        20 => {
                            base = 36;
                            NAVIGATION_SIGNAL.send(EventType::MenuDown).await;
                        },
                        //C3
                        22 => {
                            base = 48;
                        }
                        //C4
                        23 => {
                            base = 60;
                             NAVIGATION_SIGNAL.send(EventType::MenuUp).await;
                        }
                        _ => {}
                    }
                };
                LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: COLOR,
                        id: 2 + event.id,
                        brightness: BRIGTHNESS
                    })
                    .await;
            }
            EventType::KeypadUp => {
                let remap = REMAP_MIDI[event.id as usize];
                if remap != 255 {
                    let message = midi_msg::MidiMsg::ChannelVoice {
                        channel: Channel::Ch1,
                        msg: midi_msg::ChannelVoiceMsg::NoteOff {
                            note: base + remap,
                            velocity: VEL,
                        },
                    };
                    sender.publish(message).await;
                };
                LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: colors::BLACK,
                        id: 2 + event.id,
                         brightness: BRIGTHNESS
                    })
                    .await;
            }
            (e) => {
                NAVIGATION_SIGNAL.send(e).await;
            }
        }
    }
}
