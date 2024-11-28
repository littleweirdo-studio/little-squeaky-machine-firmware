#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy::executor::Executor;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_println::println;
// ESP WIFI
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Config, Ipv4Address, Stack, StackResources};
use embassy_time::{Duration, Timer};
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_wifi::wifi::{WifiController, WifiDevice, WifiEvent, WifiStaDevice, WifiState};
// Hardware Abstraction Layer
use esp_hal as hal;
use hal::clock;
pub type BootButton = crate::hal::gpio::Gpio0<crate::hal::gpio::Input<crate::hal::gpio::Pull>>;
use hal::clock::ClockControl;
use hal::peripherals::UART0;
use hal::prelude::nb::block;
use hal::rng;
use hal::{
    cpu_control, cpu_control::CpuControl, embassy, peripherals::Peripherals, prelude::*,
    timer::TimerGroup,
};

use heapless::String;
use static_cell::make_static;

use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel as Chn, signal::Signal,
};

const SSID: &str = "foldingbeauty";
const PASSWORD: &str = "Thx1138VanGils";
pub const UDP_BUFFER_SIZE: usize = 128;
static mut APP_CORE_STACK: cpu_control::Stack<32000> = cpu_control::Stack::new();

mod midi;
use midi::MidiMessage;

mod rtp_midi;
use rtp_midi::AppleMIDIReader;
use rtp_midi::RtpReader;
use rtp_midi::{AppleMIDICommand, RPT_MIDI_PAYLOAD};

mod synth;

#[main]
async fn main(spawner: Spawner) -> ! {
    // setup the system devices and clocking (MAX speed)
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let mut cpu_control = CpuControl::new(system.cpu_control);
    let clocks = ClockControl::max(system.clock_control).freeze();
    let timer = hal::timer::TimerGroup::new(peripherals.TIMG1, &clocks).timer0;
    let mut rng = Rng::new(peripherals.RNG);
    let rtc = Rtc::new(peripherals.LPWR);
    let uart = Uart::new(peripherals.UART0, &clocks);

    println!(".");
    println!(".");
    println!(".");
    println!(".");
    println!("-:=                               =:-");
    println!("-:=  Little Squeaky Machine V1.0  =:-");
    println!("-:=     (Little Weirdo Engine)    =:-");
    println!(".");
    println!(".");
    println!(". {} .", rtc.get_time_ms());
    println!(".");

    let midi_messages_channel = &*make_static!(Chn::new());
    let app_core_main_task = move || {
        let executor = make_static!(Executor::new());
        executor.run(|spawner| {
            spawner
                .spawn(synth::synth_task(&midi_messages_channel))
                .ok();
        });
    };

    let _guard = cpu_control
        .start_app_core(unsafe { &mut APP_CORE_STACK }, app_core_main_task)
        .unwrap();

    // initalize WIFI driver
    let init_wifi = esp_wifi::initialize(
        EspWifiInitFor::Wifi,
        timer,
        rng,
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    // setup timers for async embassy tasks
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0);

    // create new WIFI devices
    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init_wifi, wifi, WifiStaDevice).unwrap();

    // config network stack with DHCP
    let config = Config::dhcpv4(Default::default());
    let seed = rng.random() as u64;

    // init network stack
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    spawner.spawn(wifi_reconsolidation_loop(controller)).ok();
    spawner.spawn(wifi_stack_loop(&stack)).ok();
    println!("-:= WIFI :: Waiting for Link Active...");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("-:= WIFI :: Waiting to get IP addressss...");

    loop {
        if let Some(config) = stack.config_v4() {
            println!("-:= WIFI :: Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    let my_ssrc = rng.random();
    let mut check = spawner.spawn(rtp_midi_message_loop(
        &stack,
        my_ssrc,
        &midi_messages_channel,
    ));
    match check {
        Ok(_) => {
            println!(
                "-:= RPT MIDI ({}):: Midi Message loop started @ 5112",
                my_ssrc
            );
        }
        Err(e) => {
            println!("-:= RPT MIDI :: Midi Message loop failed to spawn {e:?}");
        }
    }

    check = spawner.spawn(rtp_midi_control_loop(&stack, my_ssrc));
    match check {
        Ok(_) => {
            println!(
                "-:= RPT MIDI ({}):: Midi Control loop started @ 5111",
                my_ssrc
            );
        }
        Err(e) => {
            println!("-:= RPT MIDI :: Midi Control loop failed to spawn {e:?}");
        }
    }

    // spawner.spawn(console_loop(uart));

    //preventing to fall in sleep
    loop {
        Timer::after(Duration::from_millis(5000)).await;
    }
}

#[embassy_executor::task]
async fn wifi_reconsolidation_loop(mut controller: WifiController<'static>) {
    println!(
        "-:= WIFI :: Device capabilities: {:?}",
        controller.get_capabilities()
    );
    loop {
        match esp_wifi::wifi::get_wifi_state() {
            WifiState::StaConnected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: String::try_from(SSID).unwrap(),
                password: String::try_from(PASSWORD).unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("-:= WIFI :: Starting wifi");
            controller.start().await.unwrap();
            println!("-:= WIFI :: Wifi started!");
        }
        println!("-:= WIFI :: About to connect...");

        match controller.connect().await {
            Ok(_) => {
                println!("-:= WIFI :: Wifi connected!");
                break;
            }
            Err(e) => {
                println!("-:= WIFI :: Failed to connect to wifi (retry in 5 seconds): {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn wifi_stack_loop(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}

#[embassy_executor::task]
async fn rtp_midi_message_loop(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    own_ssrc: u32,
    midi_message_send_channel: &'static Chn<CriticalSectionRawMutex, MidiMessage, 10>,
) {
    let mut receive_buf = [0; UDP_BUFFER_SIZE];
    let mut send_buf = [0; UDP_BUFFER_SIZE];
    let mut receive_meta = [PacketMetadata::EMPTY; UDP_BUFFER_SIZE];
    let mut send_meta = [PacketMetadata::EMPTY; UDP_BUFFER_SIZE];

    let mut socket = UdpSocket::new(
        &stack,
        &mut receive_meta,
        &mut receive_buf,
        &mut send_meta,
        &mut send_buf,
    );

    let mut rx_buffer = [0; UDP_BUFFER_SIZE];
    let mut tx_buffer = [0; UDP_BUFFER_SIZE];

    socket.bind(5112).unwrap();

    loop {
        if let Ok((amt, src)) = socket.recv_from(&mut rx_buffer).await {
            if let Ok(receive_apple_midi_message) = AppleMIDIReader::new(&mut rx_buffer[..amt]) {
                let initiator_token = receive_apple_midi_message.initiator_token();
                match receive_apple_midi_message.command() {
                    AppleMIDICommand::IN => {
                        let len = receive_apple_midi_message.createPacket(
                            AppleMIDICommand::OK,
                            own_ssrc,
                            initiator_token,
                            &mut tx_buffer,
                            0,
                        );
                        let _ = socket.send_to(&mut tx_buffer[0..len], src).await;
                    }
                    AppleMIDICommand::CK => {
                        if receive_apple_midi_message.amount_of_timestamps() == 1 {
                            let len = receive_apple_midi_message.createPacket(
                                AppleMIDICommand::CK,
                                own_ssrc,
                                initiator_token,
                                &mut tx_buffer,
                                receive_apple_midi_message.get_timestamp(0),
                            );
                            let _ = socket.send_to(&mut tx_buffer[0..len], src).await;
                        }
                    }
                    AppleMIDICommand::BY => {}
                    _ => {}
                }
            } else {
                if let Ok(rtp) = RtpReader::new(&mut rx_buffer[..amt]) {
                    if rtp.payload_type() == RPT_MIDI_PAYLOAD {
                        let message = midi::parse_midi_payload(rtp.payload());
                        let _ = midi_message_send_channel.try_send(message);
                    }
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn rtp_midi_control_loop(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    own_ssrc: u32,
) {
    let mut receive_buf = [0; UDP_BUFFER_SIZE];
    let mut send_buf = [0; UDP_BUFFER_SIZE];
    let mut receive_meta = [PacketMetadata::EMPTY; UDP_BUFFER_SIZE];
    let mut send_meta = [PacketMetadata::EMPTY; UDP_BUFFER_SIZE];

    let mut socket = UdpSocket::new(
        &stack,
        &mut receive_meta,
        &mut receive_buf,
        &mut send_meta,
        &mut send_buf,
    );

    let mut rx_buffer = [0; UDP_BUFFER_SIZE];
    let mut tx_buffer = [0; UDP_BUFFER_SIZE];

    socket.bind(5111).unwrap();
    loop {
        if let Ok((amt, src)) = socket.recv_from(&mut rx_buffer).await {
            if let Ok(midipacket) = AppleMIDIReader::new(&mut rx_buffer[..amt]) {
                let initiator_token = midipacket.initiator_token();
                match midipacket.command() {
                    AppleMIDICommand::IN => {
                        let len = midipacket.createPacket(
                            AppleMIDICommand::OK,
                            own_ssrc,
                            initiator_token,
                            &mut tx_buffer,
                            0,
                        );
                        let _ = socket.send_to(&mut tx_buffer[0..len], src).await;
                    }
                    AppleMIDICommand::BY => {}
                    _ => {}
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn console_loop(mut uart: Uart<'static, UART0>) {
    let mut input_buffer: [u8; 128] = [0u8; 128];
    let mut input_buffer_pointer = 0;

    loop {
        let read = block!(uart.read());
        match read {
            Ok(input) => match input {
                0x0D => {
                    if "hello".as_bytes() == &input_buffer[0..input_buffer_pointer] {
                        uart.write_bytes("\nhello world!".as_bytes());
                    } else if "time".as_bytes() == &input_buffer[0..input_buffer_pointer] {
                        uart.write_bytes("\ntime {}".as_bytes());
                    } else {
                        uart.write_bytes("\ncommand \u{001b}[31m[".as_bytes());
                        uart.write_bytes(&input_buffer[0..input_buffer_pointer]);
                        uart.write_bytes("]\u{001b}[0m not found".as_bytes());
                    }
                    uart.write_bytes("\n@littleweirdo > ".as_bytes());
                    input_buffer_pointer = 0;
                }
                0x20...0x7F => {
                    //echo and store potential command
                    uart.write(input);
                    input_buffer[input_buffer_pointer] = input;
                    input_buffer_pointer += 1;
                }
                // backspace
                0x08 => {
                    if input_buffer_pointer > 0 {
                        uart.write(input);
                        uart.write(0x20);
                        uart.write(input);

                        input_buffer_pointer -= 1
                    };
                }
                _ => {
                    // ignore special chars
                }
            },
            Err(error) => {
                println!("{:?}", error)
            }
        };
    }
}
