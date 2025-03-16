use embassy_futures::join::join;
use embassy_time::{Duration, Timer};

//
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;
//
extern crate alloc;
use embassy_usb::Builder;
use esp_alloc as _;
use esp_backtrace as _;
//
use esp_hal::otg_fs::asynch::Driver;
use esp_hal::otg_fs::Usb;
use esp_hal::peripherals::Peripherals;
use esp_println::println;
use midi_msg::ReceiverContext;
//
use crate::MIDI_MESSAGE_CHANNEL;
use usbd_midi::packet::reader::UsbMidiPacketReader;

#[embassy_executor::task]
pub async fn midi_usb_task() {
    let usb;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    }

    let mut ep_out_buffer = [0u8; 1024];
    let config = esp_hal::otg_fs::asynch::Config::default();
    let driver = Driver::new(usb, &mut ep_out_buffer, config);
    // Setup the configuration for the USB device
    let mut config = embassy_usb::Config::new(0x303A, 0x3001);
    config.manufacturer = Some("Hi Squeaky Things");
    config.product = Some("Lttl Squeak Machine");
    config.serial_number = Some("00110022");
    config.device_class = 0x00;
    config.device_sub_class = 0x00;
    config.device_protocol = 0x00;
    config.composite_with_iads = false;
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    // initlize the actual MIDI class
    let mut midi_class = MidiClass::new(&mut builder, 1, 1, 64);
    let mut usb_device = builder.build();
    let usb_future = usb_device.run();
    Timer::after(Duration::from_millis(100)).await;
    let process_midi = async {
        loop {
            midi_class.wait_connection().await;
            let _ = midi_read(&mut midi_class).await;
        }
    };
    join(usb_future, process_midi).await;
}

pub async fn midi_read<'d>(class: &mut MidiClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    let mut context = ReceiverContext::new();

    loop {
        let n = class.read_packet(&mut buf).await?;
        let buffer_reader = UsbMidiPacketReader::new(&buf, n);
        for packet in buffer_reader.into_iter() {
            if let Ok(packet) = packet {
                let parsed_message = midi_msg::MidiMsg::from_midi_with_context_no_extensions(
                    packet.payload_bytes(),
                    &mut context,
                );
                match parsed_message {
                    Ok((message, _size_of_processed_bytes)) => {
                        MIDI_MESSAGE_CHANNEL.send(message).await
                    }
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
