
use driver_stk8321::STK8321;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex};
use esp_backtrace as _;
use esp_println::println;
use midi_msg::Channel;

use crate::{I2c1Bus, MIDI_MESSAGE_CHANNEL};
extern crate alloc;

//pub static TILT_SIGNAL: embassy_sync::channel::Channel<CriticalSectionRawMutex, Event, 50> = embassy_sync::channel::Channel::new();


#[embassy_executor::task]
pub async fn tilt_task(i2c_bus: &'static I2c1Bus) {
 let i2c_dev = I2cDevice::new(i2c_bus);
let mut stk8321 = STK8321::new(i2c_dev);
    let ready = stk8321.ready().await;
    let _ = stk8321.init().await;
    match ready {
        Ok(true) => println!("STK8321 is ready!"),
        Ok(false) => println!("STK8321 is not ready!"),
        Err(e) => println!("Error checking if STK8321 is ready: {:?}", e),
    }
    let mut active_node:u8 = 30;
    let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();
    
    
    loop {
        /* 
       //   match TILT_SIGNAL.receive().await {
       //         EventType::TiltNoteDown => {
                     let x = stk8321.read_x().await.unwrap().abs();
                println!("X-axis value: {}", x);
              /*            LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: smart_leds::colors::ALICE_BLUE,
                        id: 0,
                        brightness: (x / 105) as u8,
                    })
                    .await;*/
                active_node = 30 + (x / 17) as u8;

                
                     let message = midi_msg::MidiMsg::ChannelVoice {
                        channel: Channel::Ch1,
                        msg: midi_msg::ChannelVoiceMsg::NoteOn {
                            note: active_node,
                            velocity: 100,
                        },
                    };
                    sender.publish(message).await;
                },
            //    EventType::TiltNoteUp => {
                     let message = midi_msg::MidiMsg::ChannelVoice {
                        channel: Channel::Ch1,
                        msg: midi_msg::ChannelVoiceMsg::NoteOff {
                            note: active_node,
                            velocity: 100,
                        },
                    };
                    sender.publish(message).await;
                },
               // EventType::KeypadLeft => active_item.back(),
               // EventType::KeypadRight => active_item.select(),
            _ => {}
        }*/

          let x = stk8321.read_x().await.unwrap().abs();

            if (x > 100) {
      
                
                println!("X-axis value: {}", x);
            }

        let y = stk8321.read_y().await.unwrap().abs();
        if (y > 100) {
                println!("Y-axis value: {}", y);
              /*            LED_MESSAGE_CHANNEL
                    .send(LedMessage {
                        color: smart_leds::colors::ALICE_BLUE,
                        id: 0,
                        brightness: (x / 105) as u8,
                    })
                    .await;*/
                
            }
    
    

    Timer::after(Duration::from_millis(100)).await;
    }
}