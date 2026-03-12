// Import required crates and modules
use embassy_futures::join::join;
use embassy_time::{Duration, Timer};

// USB and MIDI related imports
use embassy_usb::class::midi::MidiClass;
use embassy_usb::driver::EndpointError;

// External crates
extern crate alloc;
use embassy_usb::Builder;
use esp_alloc as _;
use esp_backtrace as _;

// ESP-HAL specific imports
use esp_hal::otg_fs::Usb;
use esp_hal::otg_fs::asynch::Driver;
use esp_hal::peripherals::Peripherals;
use esp_println::println;
use midi_msg::{Channel, ReceiverContext};

// Custom module imports
use crate::MIDI_MESSAGE_CHANNEL;
use usbd_midi::packet::reader::UsbMidiPacketReader;

/// Main task for handling MIDI communication over USB
///
/// This task initializes and manages USB MIDI class communication:
/// - Sets up USB peripheral with proper configuration
/// - Creates MIDI class for USB communication
/// - Handles USB connection and MIDI packet reading
/// - Parses incoming MIDI messages and forwards them to the main channel
#[embassy_executor::task]
pub async fn midi_usb_task() {
    // Initialize USB peripheral using GPIO20 and GPIO19
    let usb;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    }

    // Buffer for USB OUT endpoint
    let mut ep_out_buffer = [0u8; 1024];

    // Configure USB driver
    let config = esp_hal::otg_fs::asynch::Config::default();
    let driver = Driver::new(usb, &mut ep_out_buffer, config);

    // Setup USB device configuration
    let mut config = embassy_usb::Config::new(0x303A, 0x3001); // Vendor ID and Product ID
    config.manufacturer = Some("Hi Squeaky Things"); // Device manufacturer
    config.product = Some("Little Squeaky Machine"); // Device product name
    config.serial_number = Some("LSM0000001"); // Device serial number
    config.device_class = 0x00; // USB device class
    config.device_sub_class = 0x00; // USB device sub-class
    config.device_protocol = 0x00; // USB device protocol
    config.composite_with_iads = false; // Not using IADs

    // Buffers for USB descriptors and control
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    // Create USB device builder
    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    // Initialize the actual MIDI class with 1 IN and 1 OUT endpoint, 64-byte packet size
    let mut midi_class = MidiClass::new(&mut builder, 1, 1, 64);

    // Build the USB device
    let mut usb_device = builder.build();

    // Run the USB device in background
    let usb_future = usb_device.run();

    // Wait a short time for USB to initialize
    Timer::after(Duration::from_millis(100)).await;

    // Process MIDI packets in background
    let process_midi = async {
        loop {
            // Wait for USB connection
            midi_class.wait_connection().await;

            // Read and process MIDI packets
            let _ = midi_read(&mut midi_class).await;
        }
    };

    // Run both USB processing and MIDI reading concurrently
    join(usb_future, process_midi).await;
}

/// Read and process MIDI packets from USB
///
/// This function continuously reads MIDI packets from the USB MIDI class,
/// parses them using the midi_msg library, and sends valid messages to the
/// main MIDI message channel.
///
/// # Arguments
/// * `class` - Mutable reference to the MIDI class instance
///
/// # Returns
/// * `Result<(), Disconnected>` - Success or disconnection error
pub async fn midi_read<'d>(class: &mut MidiClass<'d, Driver<'d>>) -> Result<(), Disconnected> {
    // Buffer for reading MIDI packets
    let mut buf = [0; 64];

    // Context for MIDI message parsing
    let mut context = ReceiverContext::new();
  let sender = MIDI_MESSAGE_CHANNEL.publisher().unwrap();

    loop {
        // Read a single MIDI packet from USB
        let n = class.read_packet(&mut buf).await?;

        // Create a reader for USB MIDI packets
        let buffer_reader = UsbMidiPacketReader::new(&buf, n);

        // Process each MIDI packet in the buffer
        for packet in buffer_reader.into_iter().flatten() {
            // Parse the MIDI message with context
            let parsed_message = midi_msg::MidiMsg::from_midi_with_context_no_extensions(
                packet.payload_bytes(), // Get payload bytes from packet
                &mut context,           // Use the context for parsing
            );

            // Handle parsed message or error
            match parsed_message {
                Ok((message, _size_of_processed_bytes)) =>
                // Send valid MIDI message to main channel
                {
                    sender.publish(message.clone()).await;
                }
                Err(e) => println!("Error = {:?}", e),
            }
        }
    }
}

/// Error type for USB disconnection
///
/// Used to represent disconnection events from USB endpoints
pub struct Disconnected {}

/// Implementation of From trait to convert EndpointError to Disconnected
impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"), // Panic on buffer overflow
            EndpointError::Disabled => Disconnected {}, // Return empty disconnected error
        }
    }
}
