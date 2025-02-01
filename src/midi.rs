//
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;
//
extern crate alloc;
use esp_alloc as _;
use esp_backtrace as _;
//
use esp_hal::otg_fs::asynch::Driver;
use esp_println::println;
//
use usbd_midi::packet::reader::UsbMidiPacketReader;
use crate::MIDI_MESSAGE_CHANNEL;

pub async fn midi_read<'d>(class: &mut MidiClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let buffer_reader = UsbMidiPacketReader::new(&buf, n);
        for packet in buffer_reader.into_iter() {
            if let Ok(packet) = packet {
                let message = usbd_midi::message::Message::try_from(packet.payload_bytes());
                match message {
                    Ok(m) =>  MIDI_MESSAGE_CHANNEL.send(m).await,
                    Err(e) => println!("Error = {:?}", e),
                }
                }
            }
        }
    }

pub struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}