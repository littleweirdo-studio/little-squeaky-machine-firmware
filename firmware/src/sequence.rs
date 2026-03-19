use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use embassy_time::{Duration, Timer, WithTimeout};
use esp_backtrace as _;
use esp_println::{print, println};
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

    sequencer_1.set_lane_note(0, 36); // Kick drum
    sequencer_1.set_lane_note(1, 37); // Kick drum
    sequencer_1.set_lane_note(2, 38); // Kick drum
    sequencer_1.set_lane_note(3, 39); // Kick drum
    sequencer_1.set_lane_note(4, 41); // Kick drum
    sequencer_1.set_lane_note(5, 42); // Kick drum
    sequencer_1.set_lane_note(6, 43); // Kick drum
    sequencer_1.set_lane_note(7, 44); // Kick drum
    sequencer_1.set_lane_note(8, 45); // Kick drum
    sequencer_1.set_lane_note(9, 46); // Kick drum

    
    sequencer_1.start();

    for i in 0..15 {
        LED_MESSAGE_CHANNEL
            .send(LedMessage {
                color: smart_leds::colors::GREEN,
                id: ID_LED_MAP[i as usize],
                brightness: 10,
            })
            .await;
    }

    let mut last_id = sequencer_1.counter;
    let mut active_lane = 0;

    let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();
    loop {
        Timer::after(Duration::from_millis(CLOCK_TICK as u64)).await;
        let seq_key = SEQUENCE_KEY_CHANNEL.try_receive();
        match seq_key {
            Ok(e) => match e.event {
                EventType::ChangeSeqKey => {
                    let state = sequencer_1.flip_step(e.value as usize, e.id as usize);
                     LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: if state { smart_leds::colors::RED } else { smart_leds::colors::GREEN }, 
                                id: ID_LED_MAP[e.id as usize],
                                brightness: 10,
                            })
                            .await;
                }
                EventType::ChangeSeqLane => {
                    active_lane = e.value;
                 //   println!("active lane = {}", active_lane);
                    let lane = sequencer_1.get_lane(active_lane as usize).steps;
                    for i in 0..lane.len() {
                        LED_MESSAGE_CHANNEL
                            .send(LedMessage {
                                color: if lane[i] { smart_leds::colors::RED } else { smart_leds::colors::GREEN }, 
                                id: ID_LED_MAP[i as usize],
                                brightness: 10,
                            })
                            .await;
                    }
                }
                _ => {}
            },
            Err(_) => {}
        }

        let trigger_1 = sequencer_1.clock(); // Update the sequencer state (not currently used to trigger anything)

        for (i, &hit) in trigger_1.iter().enumerate() {
            if hit.0 {

                if i == active_lane as usize {
                LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: smart_leds::colors::BLUE,
                        id: ID_LED_MAP[sequencer_1.counter as usize],
                        brightness: 10,
                    })
                    .await;
                }
                let message = MidiMsg::ChannelVoice {
                    channel: midi_msg::Channel::Ch2,
                    msg: midi_msg::ChannelVoiceMsg::NoteOn {
                        note: hit.1,
                        velocity: 127,
                    },
                };
                let _ = sender.publish(message).await;
            } else {
                    if i == active_lane as usize {
                LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: smart_leds::colors::DARK_GREEN,
                        id: ID_LED_MAP[sequencer_1.counter as usize],
                        brightness: 10,
                    })
                    .await;
                }
            }

            //Timer::after(Duration::from_micros(1)).await;
        }
    }
}
