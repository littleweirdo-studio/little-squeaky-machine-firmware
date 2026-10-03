use embassy_executor::SendSpawner;
use esp_println::println;
use little_weirdo::sequencer::{AMOUNT_OF_LANES, MAX_AMOUNT_OF_PATTERNS};
use little_weirdo_gui::{Event, MAPPING_SIGNAL, NAVIGATION_SIGNAL};
use midi_msg::Channel;
use smart_leds::{RGB8, colors};

use crate::{
    MIDI_MESSAGE_CHANNEL, MIDI_OUT_MESSAGE_CHANNEL,
    codec::{CODEC_SIGNAL, CodecEvent, CodecEventType},
    keyboard_input,
    led::{LED_MESSAGE_CHANNEL, LedMessage},
    sequence::sequence_task,
};

const COLOR: RGB8 = colors::DEEP_PINK;

#[rustfmt::skip]
const REMAP_MIDI: [u8; 25] = [
    99, 00, 08, 09, 17, //00..04
    01, 02, 10, 11, 19, //05..09
    03, 04, 99, 12, 99, //10..14
    99, 05, 13, 14, 18, //15..19
    06, 07, 15, 16, 99, //20..24
];

#[rustfmt::skip]
const REMAP_MIDI_LED: [u8; 25] = [
    99, 07, 12, 17, 22, //00..04
    03, 08, 13, 18, 23, //05..09
    04, 09, 99, 19, 99, //10..14
    99, 10, 15, 20, 25, //15..19
    06, 11, 16, 21, 99, //20..24
];

#[rustfmt::skip]
const REMAP_SEQ: [u8; 25] = [
    00, 01, 02, 03, 04, 
    08, 09, 10, 11, 12, 
    05, 06, 07, 99, 99, 
    13, 14, 15, 99, 99, 
    99, 99, 99, 99, 99,
];

const VEL: u8 = 70;
// const BRIGTHNESS: u8 = 255;
const MARGIN: i16 = 2;

const DEBUG: bool = false;

#[embassy_executor::task]
pub async fn mapping_task() {
    let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();
    //C4
    let mut base = 35 + 24;
    let mut synth_active = true;
    let mut seq_active = false;
    let mut last_value_pot0: u16 = 300;
    let mut last_value_pot1: u16 = 300;
    let mut last_value_pot2: u16 = 300;
    let mut last_value_pot3: u16 = 300;

    let mut lane = 0;
    let mut pattern = 0;

    loop {
        let event: Event = MAPPING_SIGNAL.receive().await;
        if DEBUG {
            println!(
                "event {:?} = {:?}, lengt = {:?} ",
                event.event_identifier,
                event.id,
                MAPPING_SIGNAL.len()
            );
        };
        match event.event_identifier {
            /*
                        EventType::PotMeterChange => {
                            //println!("potmeter changed = {}, {}", event.id, event.value );
                            if event.id == 0 {
                                if last_value_pot0 != event.value
                                    && (event.value as i16 - last_value_pot0 as i16).abs() > MARGIN
                                {
                                    last_value_pot0 = event.value;

                                    NAVIGATION_SIGNAL
                                        .send(KeyEvent {
                                            event: EventType::PotMeterChange,
                                            id: 0,
                                            value: event.value,
                                        })
                                        .await;

                                    CODEC_SIGNAL
                                        .send(CodecEvent {
                                            event: CodecEventType::ChangeVolume,
                                            value: event.value,
                                        })
                                        .await;
                                }
                            }
                            if event.id == 1 {
                                if last_value_pot1 != event.value
                                    && (event.value as i16 - last_value_pot1 as i16).abs() > MARGIN
                                {
                                    last_value_pot1 = event.value;
                                    NAVIGATION_SIGNAL
                                        .send(KeyEvent {
                                            event: EventType::PotMeterChange,
                                            id: 1,
                                            value: event.value,
                                        })
                                        .await;
                                    let message = midi_msg::MidiMsg::ChannelVoice {
                                        channel: if synth_active {
                                            Channel::Ch1
                                        } else {
                                            Channel::Ch2
                                        },
                                        msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                            control: midi_msg::ControlChange::AttackTime(event.value as u8),
                                        },
                                    };

                                    sender.publish(message).await;
                                }
                            }
                            if event.id == 2 {
                                if last_value_pot2 != event.value
                                    && (event.value as i16 - last_value_pot2 as i16).abs() > MARGIN
                                {
                                    last_value_pot2 = event.value;
                                    NAVIGATION_SIGNAL
                                        .send(KeyEvent {
                                            event: EventType::PotMeterChange,
                                            id: 2,
                                            value: event.value,
                                        })
                                        .await;
                                    if seq_active {
                                        SEQUENCE_KEY_CHANNEL
                                            .send(KeyEvent {
                                                event: EventType::ChangeSeqBPM,
                                                id: 0,
                                                value: event.value,
                                            })
                                            .await;
                                    }
                                    let message = midi_msg::MidiMsg::ChannelVoice {
                                        channel: if synth_active {
                                            Channel::Ch1
                                        } else {
                                            Channel::Ch2
                                        },
                                        msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                            control: midi_msg::ControlChange::DecayTime(event.value as u8),
                                        },
                                    };

                                    sender.publish(message).await;
                                }
                            }
                            if event.id == 3 {
                                if last_value_pot3 != event.value
                                    && (event.value as i16 - last_value_pot3 as i16).abs() > MARGIN
                                {
                                    last_value_pot3 = event.value;
                                    NAVIGATION_SIGNAL
                                        .send(KeyEvent {
                                            event: EventType::PotMeterChange,
                                            id: 3,
                                            value: event.value,
                                        })
                                        .await;

                                    let message = midi_msg::MidiMsg::ChannelVoice {
                                        channel: if synth_active {
                                            Channel::Ch1
                                        } else {
                                            Channel::Ch2
                                        },
                                        msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                            control: midi_msg::ControlChange::ReleaseTime(event.value as u8),
                                        },
                                    };

                                    sender.publish(message).await;
                                }
                            }
                        }
                        EventType::DrumModeActive => {
                            //println!("drum activated");
                            synth_active = false;
                            base = 35;
                        }
                        EventType::SynthModeActive => {
                            //println!("synth activated");
                            synth_active = true;
                        }
                        EventType::SequencerModeActive => {
                            seq_active = true;
                            let seq_task = sequence_task().unwrap();
                            SendSpawner::for_current_executor().await.spawn(seq_task);
                        }
                        EventType::PatchSelect => {
                            //println!("patch activated = {}", event.id);
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
            */
            keyboard_input::KEYPAD_EVENT_DOWN => {
                let mut remap = REMAP_MIDI[event.id as usize];
                let mut led_remap = REMAP_MIDI_LED[event.id as usize];

                if seq_active {
                    remap = REMAP_SEQ[event.id as usize];
                }

                if remap != 99 {
                    if !seq_active {
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
                        //     MIDI_OUT_MESSAGE_CHANNEL.send(message.clone()).await;
                        sender.publish(message).await;
                    } else {
                        /*    SEQUENCE_KEY_CHANNEL
                        .send(KeyEvent {
                            event: EventType::ChangeSeqKey,
                            id: remap,
                            value: lane,
                        })
                        .await;*/
                    }
                } else {
                    println!("no node {}", event.id);
                    match event.id {
                        13 => {
                            if seq_active {
                                if pattern > 0 {
                                    pattern = pattern - 1;
                                    println!("pattern {}", pattern);
                                    /*
                                    SEQUENCE_KEY_CHANNEL
                                        .send(KeyEvent {
                                            event: EventType::ChangeSeqPattern,
                                            id: 0,
                                            value: pattern,
                                        })
                                        .await;*/
                                }
                            } else {
                                //  if synth_active {
                                // base -= 12;
                                // }
                            }
                        }
                        14 => {
                            //if synth_active { base += 12; }
                            if seq_active {
                                if pattern < (MAX_AMOUNT_OF_PATTERNS - 1) as u16 {
                                    pattern = pattern + 1;
                                    println!("pattern {}", pattern);
                                    /*
                                    SEQUENCE_KEY_CHANNEL
                                        .send(KeyEvent {
                                            event: EventType::ChangeSeqPattern,
                                            id: 0,
                                            value: pattern,
                                        })
                                        .await;
                                    */
                                }
                            } else {
                                //  base += 12;
                            }
                        }
                        18 => {
                            if seq_active {
                                /*
                                SEQUENCE_KEY_CHANNEL
                                    .send(KeyEvent {
                                        event: EventType::PlayActiveLaneNoteSeq,
                                        id: 0,
                                        value: 0,
                                    })
                                    .await;
                                */
                            }
                        }
                        23 => {
                            if seq_active {
                                /*
                                SEQUENCE_KEY_CHANNEL
                                    .send(KeyEvent {
                                        event: EventType::StartStopSeqPattern,
                                        id: 0,
                                        value: 0,
                                    })
                                    .await;
                                */
                            }

                            //TILT_SIGNAL.send(EventType::TiltNoteDown).await;
                        }
                        19 => {
                            /*
                            NAVIGATION_SIGNAL
                                .send(KeyEvent {
                                    event: EventType::MenuLeft,
                                    id: 0,
                                    value: 0,
                                })
                                .await;
                            */
                            //TILT_SIGNAL.send(EventType::TiltNoteDown).await;
                        }
                        //C2
                        20 => {
                            //   base = 36;
                            if seq_active {
                                if lane > 0 {
                                    lane = lane - 1;
                                    /*
                                    SEQUENCE_KEY_CHANNEL
                                        .send(KeyEvent {
                                            event: EventType::ChangeSeqLane,
                                            id: 0,
                                            value: lane,
                                        })
                                        .await;
                                    */
                                }
                            }
                            /*
                            NAVIGATION_SIGNAL
                                .send(KeyEvent {
                                    event: EventType::MenuUp,
                                    id: 0,
                                    value: 0,
                                })
                                .await;
                            */
                        }
                        //C3
                        21 => {
                            //    base = 48;
                            /*
                            NAVIGATION_SIGNAL
                                .send(KeyEvent {
                                    event: EventType::MenuRight,
                                    id: 0,
                                    value: 0,
                                })
                                .await;
                             */
                        }
                        //C4
                        22 => {
                            // base = 60;
                            /*
                            if seq_active {
                                if lane < (AMOUNT_OF_LANES - 1) as u16 {
                                    lane = lane + 1;
                                    SEQUENCE_KEY_CHANNEL
                                        .send(KeyEvent {
                                            event: EventType::ChangeSeqLane,
                                            id: 0,
                                            value: lane,
                                        })
                                        .await;
                                }
                            }

                            NAVIGATION_SIGNAL
                                .send(KeyEvent {
                                    event: EventType::MenuDown,
                                    id: 0,
                                    value: 0,
                                })
                                .await;
                            */
                        }
                        _ => {}
                    }
                };
                if !seq_active {
                    if led_remap != 99 {
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: COLOR,
                                id: led_remap,
                                //  brightness: BRIGTHNESS,
                            })
                            .await;
                    }
                }
            }

            keyboard_input::KEYPAD_EVENT_UP => {
                let remap = REMAP_MIDI[event.id as usize];
                let mut led_remap = REMAP_MIDI_LED[event.id as usize];

                if remap != 99 {
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
                    // MIDI_OUT_MESSAGE_CHANNEL.send(message.clone()).await;
                    sender.publish(message).await;
                } else {
                    // println!("event {:?}", event.id);
                    match event.id {
                        23 => {
                            // TILT_SIGNAL.send(EventType::TiltNoteUp).await;
                        }
                        _ => {}
                    }
                };
                if !seq_active {
                    if led_remap != 99 {
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: colors::BLACK,
                                id: led_remap,
                                //brightness: BRIGTHNESS,
                            })
                            .await;
                    }
                }
            }
            e => {
                /*
                NAVIGATION_SIGNAL
                    .send(KeyEvent {
                        event: e,
                        id: 0,
                        value: 0,
                    })
                    .await;
                  */
            }
        }
    }
}
