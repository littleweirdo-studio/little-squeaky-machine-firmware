//
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;
//
extern crate alloc;
use esp_alloc as _;
use esp_backtrace as _;
//
use esp_hal::otg_fs::asynch::Driver;
//
use usbd_midi::data::usb_midi::midi_packet_reader::MidiPacketBufferReader;
use crate::MIDI_MESSAGE_CHANNEL;

pub async fn midi_read<'d>(class: &mut MidiClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let buffer_reader = MidiPacketBufferReader::new(&buf, n);
        for packet in buffer_reader.into_iter() {
            if let Ok(packet) = packet {
                match packet.message {
                    _ => {
                        MIDI_MESSAGE_CHANNEL.send(packet.message).await;
                    }
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