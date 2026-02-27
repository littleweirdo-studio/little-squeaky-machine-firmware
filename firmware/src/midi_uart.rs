// Import required crates and modules
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

// Channel for receiving MIDI messages from the UART
use crate::{MIDI_MESSAGE_CHANNEL, MIDI_OUT_MESSAGE_CHANNEL};

// Constants
const READ_BUF_SIZE: usize = 0x3FF; // Maximum buffer size for reading MIDI data

/// Main task for handling MIDI communication over UART
///
/// This task manages bidirectional MIDI communication:
/// - Reads incoming MIDI messages from UART1 (GPIO18 RX, GPIO17 TX)
/// - Sends outgoing MIDI messages to UART1
///
/// The task uses a dual-future approach with embassy_futures::select
/// to handle both incoming and outgoing MIDI data concurrently.
#[embassy_executor::task]
pub async fn midi_uart_task() {
    /*
    UART Pin Configuration:
    | Pin | 17 |  U1TXD     | 🎹 MIDI out |
    | Pin | 18 |  U1RXD     | 🎹 MIDI in |
     */

    // Initialize UART1 peripheral with MIDI-specific configuration
    let mut midi_uart;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        midi_uart = Uart::new(
            peripherals.UART1,
            Config::default()
                .with_baudrate(31_250) // Standard MIDI baud rate
                .with_data_bits(esp_hal::uart::DataBits::_8) // 8-bit data
                .with_stop_bits(esp_hal::uart::StopBits::_1), // 1 stop bit
        )
        .unwrap()
        .with_rx(peripherals.GPIO18) // GPIO18 as RX pin
        .with_tx(peripherals.GPIO17) // GPIO17 as TX pin
        .into_async(); // Convert to async mode
    }

    // Buffer for reading MIDI data
    const MAX_BUFFER_SIZE: usize = 10 * READ_BUF_SIZE + 16;
    let mut buffer: [u8; MAX_BUFFER_SIZE] = [0; MAX_BUFFER_SIZE];

    // Buffer to store incomplete MIDI messages that span multiple reads
    let mut extra_buffer = Vec::<u8>::new();

    // Context for MIDI message parsing
    let mut context = ReceiverContext::new();

    //
    let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();

    loop {
        // Create futures for reading MIDI data and receiving outgoing messages
        let midi_in_future = midi_uart.read_async(&mut buffer);
        let midi_out_future = MIDI_OUT_MESSAGE_CHANNEL.receive();

        // Wait for either incoming MIDI data OR outgoing MIDI message
        match select(midi_in_future, midi_out_future).await {
            // Handle incoming MIDI data
            Either::First(res) => match res {
                Ok(size_of_received_bytes) => {
                    // Add received bytes to our buffer
                    extra_buffer.extend_from_slice(&buffer[..size_of_received_bytes]);

                    // Process all complete MIDI messages from the buffer
                    let message_raw = extra_buffer.as_slice();
                    let mut offset = 0;
                    while let Ok((message, size_of_processed_bytes)) =
                        midi_msg::MidiMsg::from_midi_with_context_no_extensions(
                            &message_raw[offset..],
                            &mut context,
                        )
                    {
                        // Send parsed MIDI message to the main processing channels
                        sender.publish(message).await;
                        offset += size_of_processed_bytes;
                    }
                    // Remove processed bytes from the buffer
                    extra_buffer.drain(..offset);
                }
                Err(e) => println!("read error = {}", e),
            },
            // Handle outgoing MIDI message
            Either::Second(message) => {
                // Write the MIDI message to UART
                let buffer_write_success = midi_uart.write_async(&message.to_midi()).await;
                match buffer_write_success {
                    Ok(_size) => {
                        // Flush the UART buffer to ensure immediate transmission
                        let flush = midi_uart.flush_async().await;
                        match flush {
                            Ok(_) => {}
                            Err(error) => println!("write error = {}", error),
                        }
                    }
                    Err(error) => println!("write error = {}", error),
                }
            }
        }
    }
}
