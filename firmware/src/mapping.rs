use esp_println::println;
use little_weirdo_gui::{EventType, MAPPING_SIGNAL, NAVIGATION_SIGNAL};
use midi_msg::Channel;
use smart_leds::{RGB8, colors};

use crate::{
    MIDI_MESSAGE_CHANNEL, MIDI_OUT_MESSAGE_CHANNEL, led::{LED_MESSAGE_CHANNEL, LedMessage}, tilt::TILT_SIGNAL
};

const COLOR: RGB8 = colors::DEEP_PINK;
const REMAP_MIDI: [u8; 25] = [
    255, 2, 4, 255, 7, 1, 3, 5, 6, 8, 9, 11, 255, 255, 255, 10, 12, 13, 15, 255, 255, 255, 255, 255,
    255,
];

const VEL: u8 = 70;
const BRIGTHNESS: u8 = 255;

#[embassy_executor::task]
pub async fn mapping_task() {
    let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();
    //C4
    let mut base = 35;
    let mut synth_active = true;

    loop {
        let event = MAPPING_SIGNAL.receive().await;
        match event.event {
            EventType::DrumModeActive => {
                println!("drum activated");
                synth_active = false;
                base = 35;
            }
            EventType::SynthModeActive => {
                println!("synth activated");
                synth_active = true;
            }
            EventType::PatchSelect => {
                println!("patch activated = {}", event.id);
                let message = midi_msg::MidiMsg::ChannelVoice {
                    channel: if synth_active {
                        Channel::Ch1
                    } else {
                        Channel::Ch2
                    },
                    msg: midi_msg::ChannelVoiceMsg::ProgramChange { program: event.id },
                };

                sender.publish(message).await;
            }

            EventType::KeypadDown => {
                let remap = REMAP_MIDI[event.id as usize];
                if remap != 255 {
                    let message = midi_msg::MidiMsg::ChannelVoice {
                        channel: if synth_active {
                            Channel::Ch1
                        } else {
                            Channel::Ch2
                        },
                        msg: midi_msg::ChannelVoiceMsg::NoteOn {
                            note: base + remap,
                            velocity: VEL,
                        },
                    };
                    MIDI_OUT_MESSAGE_CHANNEL.send(message.clone()).await;
                    sender.publish(message).await;
                } else {
                    println!("no node {}", event.id);
                    match event.id {
                        13 => {
                          //  if synth_active { 
                                base -= 12; 
                           // }
                        },
                        14 => {
                            //if synth_active { base += 12; }
                            base += 12;
                        },
                        23 => {
                            TILT_SIGNAL.send(EventType::TiltNoteDown).await;
                        }
                        19 => {
                            NAVIGATION_SIGNAL.send(EventType::MenuLeft).await;
                            //TILT_SIGNAL.send(EventType::TiltNoteDown).await;
                        }
                        //C2
                        20 => {
                         //   base = 36;
                            NAVIGATION_SIGNAL.send(EventType::MenuUp).await;
                        }
                        //C3
                        21 => {
                        //    base = 48;
                            NAVIGATION_SIGNAL.send(EventType::MenuRight).await;
                        }
                        //C4
                        22 => {
                           // base = 60;
                            NAVIGATION_SIGNAL.send(EventType::MenuDown).await;
                        }
                        _ => {}
                    }
                };
                LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: COLOR,
                        id: 2 + event.id,
                        brightness: BRIGTHNESS,
                    })
                    .await;
            }
            EventType::KeypadUp => {
                let remap = REMAP_MIDI[event.id as usize];
                if remap != 255 {
                    let message = midi_msg::MidiMsg::ChannelVoice {
                        channel: if synth_active {
                            Channel::Ch1
                        } else {
                            Channel::Ch2
                        },
                        msg: midi_msg::ChannelVoiceMsg::NoteOff {
                            note: base + remap,
                            velocity: VEL,
                        },
                    };
                    MIDI_OUT_MESSAGE_CHANNEL.send(message.clone()).await;
                    sender.publish(message).await;
                } else {
                    println!("event {:?}", event.id);
                    match event.id {
                        23 => {
                            TILT_SIGNAL.send(EventType::TiltNoteUp).await;
                        }
                        _ => {}
                    }
                };
                LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: colors::BLACK,
                        id: 2 + event.id,
                        brightness: BRIGTHNESS,
                    })
                    .await;
            }
            e => {
                NAVIGATION_SIGNAL.send(e).await;
            }
        }
    }
}
