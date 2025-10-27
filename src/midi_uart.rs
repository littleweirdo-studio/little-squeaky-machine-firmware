extern crate alloc;

use alloc::vec::Vec;
use embassy_futures::select::{Either, select};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    peripherals::Peripherals,
    uart::{Config, Uart},
};
use esp_println::println;
use midi_msg::ReceiverContext;

use crate::{MIDI_MESSAGE_CHANNEL, MIDI_OUT_MESSAGE_CHANNEL};

const READ_BUF_SIZE: usize = 0x3FF;
#[embassy_executor::task]
pub async fn midi_uart_task() {
    /*
    | Pin | 17 |  U1TXD     | 🎹 MIDI out |
    | Pin | 18 |  U1RXD     | 🎹 MIDI in |
     */
    let mut midi_uart;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        midi_uart = Uart::new(
            peripherals.UART1,
            Config::default()
                .with_baudrate(31_250)
                .with_data_bits(esp_hal::uart::DataBits::_8)
                .with_stop_bits(esp_hal::uart::StopBits::_1)
        )
        .unwrap()
        .with_rx(peripherals.GPIO18)
        .with_tx(peripherals.GPIO17)
        .into_async();
    }

    const MAX_BUFFER_SIZE: usize = 10 * READ_BUF_SIZE + 16;
    let mut buffer: [u8; MAX_BUFFER_SIZE] = [0; MAX_BUFFER_SIZE];

    let mut extra_buffer = Vec::<u8>::new();
    let mut context = ReceiverContext::new();

    loop {
        let midi_in_future = midi_uart.read_async(&mut buffer);
      let midi_out_future = MIDI_OUT_MESSAGE_CHANNEL.receive();
     
     match select(midi_in_future, midi_out_future).await {
        Either::First(res) => {
        match res {
            Ok(size_of_received_bytes) => {
                extra_buffer.extend_from_slice(&buffer[..size_of_received_bytes]);
                let message_raw = extra_buffer.as_slice();
                let mut offset = 0;
                while let Ok((message, size_of_processed_bytes)) =
                    midi_msg::MidiMsg::from_midi_with_context_no_extensions(
                        &message_raw[offset..],
                        &mut context,
                    )
                {
                    println!("{:?}", message);
                    MIDI_MESSAGE_CHANNEL.send(message).await;
                    offset += size_of_processed_bytes;
                }
                extra_buffer.drain(..offset);

            }

            Err(e) => println!("read error = {}", e),
        }
    }
         Either::Second(message) => {
             let buffer_write_succes = midi_uart.write_async(&message.to_midi()).await;
             match buffer_write_succes {
                Ok(size) => {
               println!("write = {}", size);
                   let flush = midi_uart.flush_async().await;
                   match flush {
                    Ok(_) => {},
                    Err(error) => println!("write error = {}", error),
                                   }
                 },
                 Err(error) => println!("write error = {}", error),
             }
         }
        }

     }
}
