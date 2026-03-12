use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use little_weirdo_gui::{EventType};
//use embassy_time::{Duration, Timer};
use crate::{
    MIDI_MESSAGE_CHANNEL,
    led::{LED_MESSAGE_CHANNEL, LedMessage},
};

use midi_msg::MidiMsg;

const BPM: u16 = 120;
const CLOCK_TICK: u16 = 60_000 / (BPM * 4);

pub static SEQUENCE_KEY_CHANNEL:Channel<CriticalSectionRawMutex, EventType, 50> = Channel::new();

#[embassy_executor::task]
pub async fn sequence_task() {
    let mut sequencer_1 = little_weirdo::sequencer::Sequencer::new();

    // LED_MESSAGE_CHANNEL.send(LedMessage{ color: smart_leds::RGB { r: 255, g: 0, b: 0 }, id: 0 }).await;
    // LED_MESSAGE_CHANNEL.send(LedMessage{ color: smart_leds::RGB { r: 255, g: 0, b: 0 }, id: 4 }).await;
    // LED_MESSAGE_CHANNEL.send(LedMessage{ color: smart_leds::RGB { r: 255, g: 0, b: 0 }, id: 8 }).await;
    // LED_MESSAGE_CHANNEL.send(LedMessage{ color: smart_leds::RGB { r: 255, g: 0, b: 0 }, id: 12 }).await;

    sequencer_1.set_lane_note(0, 35); // Kick drum
    sequencer_1.set_lane_note(1, 36); // Kick drum
    sequencer_1.set_lane_note(2, 37); // Kick drum


    sequencer_1.start();
    //sequencer_1.set_step(0, 0);
    let mut last_id = sequencer_1.counter;

    let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();
    loop {
        Timer::after(Duration::from_millis(CLOCK_TICK as u64)).await;

        /* 
        match SEQUENCE_KEY_CHANNEL.try_receive() {
            Ok(event) => {
                match event {
                    EventType::Keypad1 => sequencer_1.flip_step(0, 0),
                    EventType::Keypad2 => sequencer_1.flip_step(0, 1),
                    EventType::Keypad3 => sequencer_1.flip_step(0, 2),
                    EventType::Keypad4 => sequencer_1.flip_step(0, 3),
                    _ => {}
                }
            },
            Err(_) => {},
        }*/

        let trigger_1 = sequencer_1.clock(); // Update the sequencer state (not currently used to trigger anything)
        LED_MESSAGE_CHANNEL
            .send(LedMessage {
                color: smart_leds::RGB { r: 0, g: 0, b: 150 },
                id: sequencer_1.counter + 2,
                   brightness: 10,
            })
            .await;
        for (i, &hit) in trigger_1.iter().enumerate() {
            if hit.0 {
                if hit.1 == 35 {
                    LED_MESSAGE_CHANNEL
                        .send(LedMessage {
                            color: smart_leds::RGB { r: 255, g: 0, b: 0 },
                            id: last_id,
                               brightness: 10,
                        })
                        .await;
                    LED_MESSAGE_CHANNEL
                        .send(LedMessage {
                            color: smart_leds::RGB {
                                r: 0,
                                g: 255,
                                b: 100,
                            },
                            id: sequencer_1.counter + 2,
                               brightness: 10,
                        })
                        .await;
                    last_id = sequencer_1.counter + 2;
                }

                //                 LED_MESSAGE_CHANNEL.send(LedMessage{ color: smart_leds::RGB { r: 255, g: 0, b: 0 }, id: 0 }).await;
                let message = MidiMsg::ChannelVoice {
                    channel: midi_msg::Channel::Ch2,
                    msg: midi_msg::ChannelVoiceMsg::NoteOn {
                        note: hit.1,
                        velocity: 127,
                    },
                };
                let _ = sender.publish(message).await;
            }
        }

        //Timer::after(Duration::from_micros(1)).await;
    }
}
