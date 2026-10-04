use embassy_executor::SendSpawner;
use esp_println::println;
use little_weirdo::sequencer::{AMOUNT_OF_LANES, MAX_AMOUNT_OF_PATTERNS};
use little_weirdo_gui::{Event, MAPPING_SIGNAL, NAVIGATION_SIGNAL};
use midi_msg::Channel;
use smart_leds::{RGB8, colors};

use crate::{
    MIDI_MESSAGE_CHANNEL, MIDI_OUT_MESSAGE_CHANNEL,
    codec::{CODEC_SIGNAL, CodecEvent, CodecEventType},
    encoder, keyboard_input,
    led::{LED_MESSAGE_CHANNEL, LedMessage},
    sequence::sequence_task,
    touch_input::{TOUCH_BUTTON_EVENT_DOWN, TOUCH_BUTTON_EVENT_UP},
};

const COLOR: RGB8 = colors::DEEP_PINK;

pub const CHORD_CHANGED_EVENT_IDENTIFIER: u8 = 248;
pub const CHORD_NAMES: [&str; 8] = [
    "Major",
    "Minor",
    "Diminished",
    "Augmented",
    "Suspended 2",
    "Suspended 4",
    "Major 7",
    "Minor 7",
];

// Touch pads 0..7 select major, minor, diminished, augmented, sus2, sus4, maj7, and min7.
const CHORD_INTERVALS: [&[u8]; 8] = [
    &[0, 4, 7],
    &[0, 3, 7],
    &[0, 3, 6],
    &[0, 4, 8],
    &[0, 2, 7],
    &[0, 5, 7],
    &[0, 4, 7, 11],
    &[0, 3, 7, 10],
];

#[derive(Clone, Copy, Default)]
struct ActiveNotes {
    notes: [u8; 4],
    len: usize,
}

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
    04, 09, 99, 19, 24, //10..14
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
pub const WAVEFORM_CHANGED_EVENT_IDENTIFIER: u8 = 250;
pub const VOLUME_CHANGED_EVENT_IDENTIFIER: u8 = 249;
pub const ENVELOPE_CHANGED_EVENT_IDENTIFIER: u8 = 247;
pub const ENVELOPE_MODE_EVENT_ID: u8 = 4;
pub const ENVELOPE_KEY_ID: u8 = 14;
pub const ENVELOPE_KEY_LED_ID: u8 = 24;
pub const INITIAL_ENVELOPE_VALUES: [u8; 4] = [20, 20, 80, 20];
pub const OVERDRIVE_CHANGED_EVENT_IDENTIFIER: u8 = 246;
pub const OVERDRIVE_MODE_EVENT_ID: u8 = 3;
pub const OVERDRIVE_KEY_ID: u8 = 15;
pub const OVERDRIVE_KEY_LED_ID: u8 = 5;
pub const OVERDRIVE_ENABLE_CONTROL: u8 = 110;
pub const OVERDRIVE_KIND_CONTROL: u8 = 111;
pub const OVERDRIVE_THRESHOLD_CONTROL: u8 = 112;
pub const OVERDRIVE_STATUS_CONTROL: u8 = 113;
pub const DELAY_CHANGED_EVENT_IDENTIFIER: u8 = 245;
pub const DELAY_MODE_EVENT_ID: u8 = 5;
pub const DELAY_KEY_ID: u8 = 12;
pub const DELAY_KEY_LED_ID: u8 = 2;
pub const DELAY_ENABLE_CONTROL: u8 = 114;
pub const DELAY_TIME_CONTROL: u8 = 115;
pub const DELAY_MIX_CONTROL: u8 = 116;
pub const DELAY_FEEDBACK_CONTROL: u8 = 117;
pub const DELAY_STATUS_CONTROL: u8 = 118;

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

    let mut volume: u8 = 50;
    let mut voice: u8 = 0;
    let mut envelope_values = INITIAL_ENVELOPE_VALUES;
    let mut envelope_edit_mode = false;
    let mut overdrive_edit_mode = false;
    let mut delay_edit_mode = false;

    let mut lane = 0;
    let mut pattern = 0;
    let mut selected_chord: Option<usize> = None;
    let mut active_notes: [Option<ActiveNotes>; 25] = [None; 25];

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
            TOUCH_BUTTON_EVENT_DOWN => {
                if (event.id as usize) < CHORD_INTERVALS.len() {
                    selected_chord = Some(event.id as usize);
                    NAVIGATION_SIGNAL
                        .send(Event {
                            event_identifier: CHORD_CHANGED_EVENT_IDENTIFIER,
                            id: event.id,
                            value: 0,
                        })
                        .await;
                }
            }
            TOUCH_BUTTON_EVENT_UP => {
                if selected_chord == Some(event.id as usize) {
                    selected_chord = None;
                    NAVIGATION_SIGNAL
                        .send(Event {
                            event_identifier: CHORD_CHANGED_EVENT_IDENTIFIER,
                            id: CHORD_NAMES.len() as u8,
                            value: 0,
                        })
                        .await;
                }
            }
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
            encoder::ENCODER_EVENT_ROTATION => {
                if envelope_edit_mode {
                    if let Some(value) = envelope_values.get_mut(event.id as usize) {
                        let maximum = if event.id == 2 { 100 } else { 127 };
                        *value = if event.value == encoder::ENCODER_CLOCKWISE {
                            value.saturating_add(1).min(maximum)
                        } else {
                            value.saturating_sub(1)
                        };

                        let control = match event.id {
                            0 => midi_msg::ControlChange::AttackTime(*value),
                            1 => midi_msg::ControlChange::DecayTime(*value),
                            2 => midi_msg::ControlChange::SoundControl3(*value),
                            3 => midi_msg::ControlChange::ReleaseTime(*value),
                            _ => unreachable!(),
                        };
                        sender
                            .publish(midi_msg::MidiMsg::ChannelVoice {
                                channel: if synth_active {
                                    Channel::Ch1
                                } else {
                                    Channel::Ch2
                                },
                                msg: midi_msg::ChannelVoiceMsg::ControlChange { control },
                            })
                            .await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: ENVELOPE_CHANGED_EVENT_IDENTIFIER,
                                id: event.id,
                                value: *value as u16,
                            })
                            .await;
                    }
                } else if overdrive_edit_mode {
                    let control = match event.id {
                            0 => Some((OVERDRIVE_ENABLE_CONTROL, 0)),
                            1 => Some((
                                OVERDRIVE_KIND_CONTROL,
                                (event.value == encoder::ENCODER_CLOCKWISE) as u8,
                            )),
                            2 => Some((
                                OVERDRIVE_THRESHOLD_CONTROL,
                                (event.value == encoder::ENCODER_CLOCKWISE) as u8,
                            )),
                            _ => None,
                    };
                    if let Some((control, value)) = control {
                            sender
                                .publish(midi_msg::MidiMsg::ChannelVoice {
                                    channel: Channel::Ch1,
                                    msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                        control: midi_msg::ControlChange::CC { control, value },
                                    },
                                })
                                .await;
                    }
                } else if delay_edit_mode {
                    let control = match event.id {
                        0 => Some((DELAY_ENABLE_CONTROL, 0)),
                        1 => Some((
                            DELAY_TIME_CONTROL,
                            (event.value == encoder::ENCODER_CLOCKWISE) as u8,
                        )),
                        2 => Some((
                            DELAY_MIX_CONTROL,
                            (event.value == encoder::ENCODER_CLOCKWISE) as u8,
                        )),
                        3 => Some((
                            DELAY_FEEDBACK_CONTROL,
                            (event.value == encoder::ENCODER_CLOCKWISE) as u8,
                        )),
                        _ => None,
                    };
                    if let Some((control, value)) = control {
                        sender
                            .publish(midi_msg::MidiMsg::ChannelVoice {
                                channel: Channel::Ch1,
                                msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                    control: midi_msg::ControlChange::CC { control, value },
                                },
                            })
                            .await;
                    }
                } else {
                    //enc 0
                    if event.id == 0 {
                        if event.value == encoder::ENCODER_CLOCKWISE {
                            volume = volume.saturating_add(5).min(127);
                        } else {
                            volume = volume.saturating_sub(5);
                        }

                        CODEC_SIGNAL
                            .send(CodecEvent {
                                event: CodecEventType::ChangeVolume,
                                value: volume as u16,
                            })
                            .await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: VOLUME_CHANGED_EVENT_IDENTIFIER,
                                id: 0,
                                value: volume as u16,
                            })
                            .await;
                    }

                    //enc 1
                    if event.id == 1 {
                        if event.value == encoder::ENCODER_CLOCKWISE {
                            voice = voice.saturating_add(1).min(25);
                        } else {
                            voice = voice.saturating_sub(1);
                        }

                        let message = midi_msg::MidiMsg::ChannelVoice {
                            channel: if synth_active {
                                Channel::Ch1
                            } else {
                                Channel::Ch2
                            },
                            msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                control: midi_msg::ControlChange::SoundControl2(voice),
                            },
                        };

                        sender.publish(message).await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: WAVEFORM_CHANGED_EVENT_IDENTIFIER,
                                id: voice,
                                value: voice as u16,
                            })
                            .await;
                    }
                }
            }
            keyboard_input::KEYPAD_EVENT_DOWN => {
                let mut remap = REMAP_MIDI[event.id as usize];
                let mut led_remap = REMAP_MIDI_LED[event.id as usize];

                if seq_active {
                    remap = REMAP_SEQ[event.id as usize];
                }

                if event.id == ENVELOPE_KEY_ID && !seq_active && synth_active {
                    envelope_edit_mode = !envelope_edit_mode;
                    if envelope_edit_mode && overdrive_edit_mode {
                        overdrive_edit_mode = false;
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: colors::BLACK,
                                id: OVERDRIVE_KEY_LED_ID,
                            })
                            .await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: OVERDRIVE_CHANGED_EVENT_IDENTIFIER,
                                id: OVERDRIVE_MODE_EVENT_ID,
                                value: 0,
                            })
                            .await;
                    }
                    if envelope_edit_mode && delay_edit_mode {
                        delay_edit_mode = false;
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: colors::BLACK,
                                id: DELAY_KEY_LED_ID,
                            })
                            .await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: DELAY_CHANGED_EVENT_IDENTIFIER,
                                id: DELAY_MODE_EVENT_ID,
                                value: 0,
                            })
                            .await;
                    }
                    NAVIGATION_SIGNAL
                        .send(Event {
                            event_identifier: ENVELOPE_CHANGED_EVENT_IDENTIFIER,
                            id: ENVELOPE_MODE_EVENT_ID,
                            value: envelope_edit_mode as u16,
                        })
                        .await;
                }

                if event.id == OVERDRIVE_KEY_ID && !seq_active && synth_active {
                    overdrive_edit_mode = !overdrive_edit_mode;
                    if overdrive_edit_mode && envelope_edit_mode {
                        envelope_edit_mode = false;
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: colors::BLACK,
                                id: ENVELOPE_KEY_LED_ID,
                            })
                            .await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: ENVELOPE_CHANGED_EVENT_IDENTIFIER,
                                id: ENVELOPE_MODE_EVENT_ID,
                                value: 0,
                            })
                            .await;
                    }
                    if overdrive_edit_mode && delay_edit_mode {
                        delay_edit_mode = false;
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: colors::BLACK,
                                id: DELAY_KEY_LED_ID,
                            })
                            .await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: DELAY_CHANGED_EVENT_IDENTIFIER,
                                id: DELAY_MODE_EVENT_ID,
                                value: 0,
                            })
                            .await;
                    }
                    NAVIGATION_SIGNAL
                        .send(Event {
                            event_identifier: OVERDRIVE_CHANGED_EVENT_IDENTIFIER,
                            id: OVERDRIVE_MODE_EVENT_ID,
                            value: overdrive_edit_mode as u16,
                        })
                        .await;
                    if overdrive_edit_mode {
                        sender
                            .publish(midi_msg::MidiMsg::ChannelVoice {
                                channel: Channel::Ch1,
                                msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                    control: midi_msg::ControlChange::CC {
                                        control: OVERDRIVE_STATUS_CONTROL,
                                        value: 0,
                                    },
                                },
                            })
                            .await;
                    }
                }

                if event.id == DELAY_KEY_ID && !seq_active && synth_active {
                    delay_edit_mode = !delay_edit_mode;
                    if delay_edit_mode && envelope_edit_mode {
                        envelope_edit_mode = false;
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: colors::BLACK,
                                id: ENVELOPE_KEY_LED_ID,
                            })
                            .await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: ENVELOPE_CHANGED_EVENT_IDENTIFIER,
                                id: ENVELOPE_MODE_EVENT_ID,
                                value: 0,
                            })
                            .await;
                    }
                    if delay_edit_mode && overdrive_edit_mode {
                        overdrive_edit_mode = false;
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: colors::BLACK,
                                id: OVERDRIVE_KEY_LED_ID,
                            })
                            .await;
                        NAVIGATION_SIGNAL
                            .send(Event {
                                event_identifier: OVERDRIVE_CHANGED_EVENT_IDENTIFIER,
                                id: OVERDRIVE_MODE_EVENT_ID,
                                value: 0,
                            })
                            .await;
                    }
                    NAVIGATION_SIGNAL
                        .send(Event {
                            event_identifier: DELAY_CHANGED_EVENT_IDENTIFIER,
                            id: DELAY_MODE_EVENT_ID,
                            value: delay_edit_mode as u16,
                        })
                        .await;
                    if delay_edit_mode {
                        sender
                            .publish(midi_msg::MidiMsg::ChannelVoice {
                                channel: Channel::Ch1,
                                msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                    control: midi_msg::ControlChange::CC {
                                        control: DELAY_STATUS_CONTROL,
                                        value: 0,
                                    },
                                },
                            })
                            .await;
                    }
                }

                if remap != 99 {
                    if !seq_active {
                        let intervals = selected_chord
                            .map(|chord| CHORD_INTERVALS[chord])
                            .unwrap_or(&[0]);
                        let mut notes = ActiveNotes::default();
                        for (index, interval) in intervals.iter().enumerate() {
                            let note = base + remap + *interval;
                            notes.notes[index] = note;
                            notes.len += 1;
                            let message = midi_msg::MidiMsg::ChannelVoice {
                                channel: if synth_active {
                                    Channel::Ch1
                                } else {
                                    Channel::Ch2
                                },
                                msg: midi_msg::ChannelVoiceMsg::NoteOn {
                                    note,
                                    velocity: VEL,
                                },
                            };
                            sender.publish(message).await;
                        }
                        active_notes[event.id as usize] = Some(notes);
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
                    if event.id == ENVELOPE_KEY_ID {
                        led_remap = ENVELOPE_KEY_LED_ID;
                    } else if event.id == OVERDRIVE_KEY_ID {
                        led_remap = OVERDRIVE_KEY_LED_ID;
                    } else if event.id == DELAY_KEY_ID {
                        led_remap = DELAY_KEY_LED_ID;
                    }
                    if led_remap != 99 {
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: if event.id == ENVELOPE_KEY_ID {
                                    if envelope_edit_mode {
                                        colors::BLUE
                                    } else {
                                        colors::BLACK
                                    }
                                } else if event.id == OVERDRIVE_KEY_ID {
                                    if overdrive_edit_mode {
                                        colors::BLUE
                                    } else {
                                        colors::BLACK
                                    }
                                } else if event.id == DELAY_KEY_ID {
                                    if delay_edit_mode {
                                        colors::BLUE
                                    } else {
                                        colors::BLACK
                                    }
                                } else {
                                    COLOR
                                },
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
                if event.id == ENVELOPE_KEY_ID && !seq_active {
                    led_remap = ENVELOPE_KEY_LED_ID;
                } else if event.id == OVERDRIVE_KEY_ID && !seq_active {
                    led_remap = OVERDRIVE_KEY_LED_ID;
                } else if event.id == DELAY_KEY_ID && !seq_active {
                    led_remap = DELAY_KEY_LED_ID;
                }

                if remap != 99 {
                    let notes = active_notes[event.id as usize].take().unwrap_or_else(|| {
                        let mut notes = ActiveNotes::default();
                        notes.notes[0] = base + remap;
                        notes.len = 1;
                        notes
                    });
                    for note in notes.notes.into_iter().take(notes.len) {
                        let message = midi_msg::MidiMsg::ChannelVoice {
                            channel: if synth_active {
                                Channel::Ch1
                            } else {
                                Channel::Ch2
                            },
                            msg: midi_msg::ChannelVoiceMsg::NoteOff {
                                note,
                                velocity: VEL,
                            },
                        };
                        sender.publish(message).await;
                    }
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
                        let keep_edit_led =
                            (event.id == ENVELOPE_KEY_ID && envelope_edit_mode)
                                || (event.id == OVERDRIVE_KEY_ID && overdrive_edit_mode)
                                || (event.id == DELAY_KEY_ID && delay_edit_mode);
                        if !keep_edit_led {
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
