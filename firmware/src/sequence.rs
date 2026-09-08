use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use embassy_time::{Duration, Instant, Timer};
use esp_backtrace as _;
use esp_println::println;
use little_weirdo::sequencer::{AMOUNT_OF_LANES, MAX_AMOUNT_OF_PATTERNS};
use little_weirdo_gui::{EventType, KeyEvent};
//use embassy_time::{Duration, Timer};
use crate::{
    MIDI_MESSAGE_CHANNEL,
    led::{LED_MESSAGE_CHANNEL, LedMessage},
};

use midi_msg::MidiMsg;

const BPM: u16 = 120;
const CLOCK_TICK: u16 = 60_000 / (BPM * 4);

pub static SEQUENCE_KEY_CHANNEL: Channel<CriticalSectionRawMutex, KeyEvent, 50> = Channel::new();

pub static ID_LED_MAP: [u8; 16] = [2, 3, 4, 5, 6, 12, 13, 14, 7, 8, 9, 10, 11, 17, 18, 19];

#[embassy_executor::task]
pub async fn sequence_task() {
    let mut sequencer_1 = little_weirdo::sequencer::Sequencer::new();
    let mut tick = CLOCK_TICK;

    // map the sequencer
    for pattern_id in 0..MAX_AMOUNT_OF_PATTERNS {
        for lane_id in 0..AMOUNT_OF_LANES {
            sequencer_1.set_lane_note(pattern_id, lane_id, 36 + lane_id as u8); // Kpattern_idck drum
        }
    }

    sequencer_1.start();

    for i in 0..15 {
        LED_MESSAGE_CHANNEL
            .send(LedMessage {
                color: smart_leds::colors::GREEN,
                id: ID_LED_MAP[i as usize],
                // brightness: 10,
            })
            .await;
    }

    let last_id = 0;
    let last_hit = false;
    let mut active_lane = 0;
    let mut active_pattern = 0;
    let mut switch = false;

    let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();
    loop {
        let start = Instant::now();
        let seq_key = SEQUENCE_KEY_CHANNEL.try_receive();
       
        match seq_key {
            Ok(e) => match e.event {
                EventType::ChangeSeqBPM => {
                    if e.value > 0  {
                        tick = 60_000 / ( e.value * 8);
                    }
                }
                EventType::StartStopSeqPattern => {
                    if sequencer_1.is_playing() {
                        sequencer_1.stop();
                    } else {
                        sequencer_1.start();
                    }
                }
                EventType::PlayActiveLaneNoteSeq => {
                    println!("active = {}", active_lane + 36);
                    let message = MidiMsg::ChannelVoice {
                    channel: midi_msg::Channel::Ch2,
                    msg: midi_msg::ChannelVoiceMsg::NoteOn {
                        note: active_lane + 36,
                        velocity: 127,
                    },
                };
                let _ = sender.publish(message).await;
                }
                EventType::ChangeSeqKey => {
                    let state =
                        sequencer_1.flip_step(e.value as usize, e.id as usize);

                  

                    LED_MESSAGE_CHANNEL
                        .send(LedMessage {
                            color: if state {
                                smart_leds::colors::RED
                            } else {
                                smart_leds::colors::GREEN
                            },
                            id: ID_LED_MAP[e.id as usize],
                            // brightness: 10,
                        })
                        .await;
                }
                EventType::ChangeSeqLane => {
                    active_lane = e.value as u8;
                    //   println!("active lane = {}", active_lane);
                    let lane = sequencer_1
                        .get_lane(active_pattern, active_lane as usize)
                        .steps;
                    for i in 0..lane.len() {
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: if lane[i] {
                                    smart_leds::colors::RED
                                } else {
                                    smart_leds::colors::GREEN
                                },
                                id: ID_LED_MAP[i as usize],
                                // brightness: 10,
                            })
                            .await;
                    }
                }
                EventType::ChangeSeqPattern => {
                    active_pattern = e.value as usize;
                    let lane = sequencer_1
                        .get_lane(active_pattern, active_lane as usize)
                        .steps;
                    for i in 0..lane.len() {
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: if lane[i] {
                                    smart_leds::colors::RED
                                } else {
                                    smart_leds::colors::GREEN
                                },
                                id: ID_LED_MAP[i as usize],
                                // brightness: 10,
                            })
                            .await;
                    }
                }
                _ => {}
            },
            Err(_) => {}
        }

        let trigger_1 = sequencer_1.clock(); // Update the sequencer state (not currently used to trigger anything)
        let note_id_active = sequencer_1.get_position() as usize;

        if note_id_active % 4 == 0 {
  LED_MESSAGE_CHANNEL
                        .send(LedMessage {
                            color: if switch {
                                smart_leds::colors::RED
                            } else {
                                smart_leds::colors::INDIAN_RED
                            },
                            id: 0,
                            // brightness: 10,
                        })
                        .await;
                     LED_MESSAGE_CHANNEL
                        .send(LedMessage {
                            color: if switch {
                                smart_leds::colors::INDIAN_RED
                            } else {
                                smart_leds::colors::RED
                            },
                            id: 1,
                            // brightness: 10,
                        })
                        .await;
                    switch = !switch;
        }

        for (i, &hit) in trigger_1.iter().enumerate() {
            if i == active_lane as usize {
                if last_id != i {
                    LED_MESSAGE_CHANNEL
                        .send(LedMessage {
                            color: if last_hit {
                                smart_leds::colors::RED
                            } else {
                                smart_leds::colors::GREEN
                            },
                            id: ID_LED_MAP[last_id],
                            // brightness: 10,
                        })
                        .await;
                }
            }

            if hit.0 {
                let message = MidiMsg::ChannelVoice {
                    channel: midi_msg::Channel::Ch2,
                    msg: midi_msg::ChannelVoiceMsg::NoteOn {
                        note: hit.1,
                        velocity: 127,
                    },
                };
                let _ = sender.publish(message).await;
            }

            //Timer::after(Duration::from_micros(1)).await;
        }
        let stop = Instant::now();
        let duration = stop.duration_since(start);
        Timer::after(Duration::from_millis(
            tick as u64 - duration.as_millis(),
        ))
        .await;
    }
}
