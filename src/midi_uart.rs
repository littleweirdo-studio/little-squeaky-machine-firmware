extern crate alloc;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    peripherals::Peripherals,
    uart::{Config, Uart},
};
use esp_println::println;
use usbd_midi::{Message, UsbMidiPacketReader};

use crate::MIDI_MESSAGE_CHANNEL;

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
                .with_stop_bits(esp_hal::uart::StopBits::_1),
        )
        .unwrap()
        .with_rx(peripherals.GPIO18)
        .with_tx(peripherals.GPIO17)
        .into_async();
    }

    let mut buffer: [u8; 64] = [0; 64];

    let note_on_message = Message::NoteOn(usbd_midi::message::Channel::Channel1, usbd_midi::message::Note::C3, usbd_midi::message::Velocity::MAX).into_packet(usbd_midi::CableNumber::Cable0);

    loop {
        let buffer_write_succes = midi_uart.write_async(note_on_message.as_raw_bytes()).await;
        
        match buffer_write_succes {
            Ok(size) => println!("write = {}", size),    
            Err(error) => println!("write error = {}", error),
        }
        let buffer_read_succes = midi_uart.read_async(&mut buffer).await;
        match buffer_read_succes {
            Ok(size) => {
                println!("read = {}", size);
                let buffer_reader = UsbMidiPacketReader::new(&buffer, size);
                for packet in buffer_reader.into_iter() {
                    if let Ok(packet) = packet {
                        let message = usbd_midi::message::Message::try_from(packet.payload_bytes());
                        match message {
                            Ok(m) => MIDI_MESSAGE_CHANNEL.send(m).await,
                            Err(e) => println!("Error = {:?}", e),
                        }
                    }
                }
            }
            Err(error) => println!("read error = {}", error),
        }
    }
}
