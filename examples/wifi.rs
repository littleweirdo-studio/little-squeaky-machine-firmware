#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use ::embassy_net::tcp::TcpSocket;
use embassy::executor::Executor;
use embassy_executor::Spawner;
use embassy_net::{Config, Ipv4Address, Stack, StackResources};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub use esp32_hal as hal;
pub type BootButton = crate::hal::gpio::Gpio0<crate::hal::gpio::Input<crate::hal::gpio::PullDown>>;
use embassy_time::{Duration, Timer};
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_backtrace as _;

use esp_println::println;
use esp_wifi::wifi::{WifiController, WifiDevice, WifiEvent, WifiStaDevice, WifiState};
use esp_wifi::{initialize, EspWifiInitFor};
use hal::clock::ClockControl;
use hal::Rng;
use hal::{
    cpu_control,
    cpu_control::CpuControl,
    dma::DmaPriority,
    embassy,
    i2s::{DataFormat, I2s, I2s0New, I2sWriteDma, NoMclk, PinsBclkWsDout, Standard},
    pdma::Dma,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
    IO,
};

use static_cell::make_static;

use little_weirdo::synth;
use little_weirdo::synth::envelope::Envelop;
use little_weirdo::synth::filter::FilterConfig;
use little_weirdo::synth::oscillator::Waveform;
use little_weirdo::synth::patch::Patch;

const SSID: &str = "foldingbeauty";
const PASSWORD: &str = "Thx1138VanGils";
static mut APP_CORE_STACK: cpu_control::Stack<32000> = cpu_control::Stack::new();

#[main]
async fn main(spawner: Spawner) -> ! {
    // esp_println::logger::init_logger(log::LevelFilter::Info);

    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timer = hal::timer::TimerGroup::new(peripherals.TIMG1, &clocks).timer0;

    let knob = &*make_static!(Signal::new());

    let cpu1_fnctn = move || {
        let executor = make_static!(Executor::new());
        executor.run(|spawner| {
            spawner.spawn(cpu1_task(&knob)).ok();
        });
    };

    let mut cpu_control = CpuControl::new(system.cpu_control);
    let _guard = cpu_control
        .start_app_core(unsafe { &mut APP_CORE_STACK }, cpu1_fnctn)
        .unwrap();

    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiStaDevice).unwrap();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timer_group0.timer0);

    let config = Config::dhcpv4(Default::default());

    let seed = 1234; // very random, very secure seed

    // Init network stack
    let stack = &*make_static!(Stack::new(
        wifi_interface,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(&stack)).ok();

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP addressss...");

    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    loop {
        Timer::after(Duration::from_millis(1_000)).await;

        let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        let remote_endpoint = (Ipv4Address::new(142, 250, 185, 115), 80);
        println!("connecting...");
        knob.signal(1);
        let r = socket.connect(remote_endpoint).await;
        if let Err(e) = r {
            println!("connect error: {:?}", e);
            continue;
        }
        println!("connected!");
        let mut buf = [0; 1024];
        loop {
            use embedded_io_async::Write;
            let r = socket
                .write_all(b"GET / HTTP/1.0\r\nHost: www.mobile-j.de\r\n\r\n")
                .await;
            if let Err(e) = r {
                println!("write error: {:?}", e);
                break;
            }
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    println!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    println!("read error: {:?}", e);
                    break;
                }
            };
            println!("{}", core::str::from_utf8(&buf[..n]).unwrap());
        }
        Timer::after(Duration::from_millis(3000)).await;
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.get_capabilities());
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
                ssid: SSID.into(),
                password: PASSWORD.into(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start().await.unwrap();
            println!("Wifi started!");
        }
        println!("About to connect...");

        match controller.connect().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}

/// Waits for a message that contains a duration, then flashes a led for that
/// duration of time.
#[embassy_executor::task]
async fn cpu1_task(knob_signal: &'static Signal<CriticalSectionRawMutex, u16>) {
    let mut tx_descriptors = [0u32; 20 * 3];
    let mut rx_descriptors = [0u32; 8 * 3];
    let mut transfer;
    let buffer: &mut [u8; 32000] = dma_buffer();

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let system = peripherals.SYSTEM.split();
        let clocks = ClockControl::max(system.clock_control).freeze();

        let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

        let dma = Dma::new(system.dma);
        let dma_channel = dma.i2s0channel;

        let i2s = I2s::new(
            peripherals.I2S0,
            NoMclk {},
            Standard::Philips,
            DataFormat::Data16Channel16,
            22500u32.Hz(),
            dma_channel.configure(
                true,
                &mut tx_descriptors,
                &mut rx_descriptors,
                DmaPriority::Priority0,
            ),
            &clocks,
        );
        let i2s_tx = i2s.i2s_tx.with_pins(PinsBclkWsDout::new(
            io.pins.gpio12,
            io.pins.gpio13,
            io.pins.gpio14,
        ));
        transfer = i2s_tx.write_dma_circular(buffer).unwrap();
    }
    let patch = Patch {
        voice_1: Waveform::Sine,
        voice_1_mix_level: 100,
        voice_2: Waveform::Square10,
        voice_2_mix_level: 80,
        lfo_1: 10,
        lfo_1_mix_level: 0,
        voice_1_env: Envelop {
            attack_time: 1000,
            decay_time: 1000,
            release_time: 1000,
            sustain_level: 50,
        },
        voice_2_env: Envelop {
            attack_time: 1000,
            decay_time: 1000,
            release_time: 1000,
            sustain_level: 50,
        },

        voice_1_detune: 0,
        voice_2_detune: 4,
        filter_config: FilterConfig {
            cutoff_frequency: 80,
            filter_on: false,
            _24db: false,
        },
        mono: true,
        main_gain: 50,
        glide: true,
        glide_rate: 20,
    };

    let mut synth: synth::Synth = synth::Synth::new(22500, patch);
    println!("Hello World - Core 1!");

    let mut filler = [0u8; 5000];
    println!("Hello World - Core 1!");
    let mut previous_knob: u16 = 0;

    loop {
        let avail = transfer.available();
        if avail > 0 {
            let avail = usize::min(5000, avail);
            for i in 0..(avail / 4) {
                let output: i16 = synth.clock_and_output();
                let bytes = output.to_be_bytes();
                filler[i * 4] = bytes[1];
                filler[i * 4 + 1] = bytes[0];
                filler[i * 4 + 2] = bytes[1];
                filler[i * 4 + 3] = bytes[0];
            }
            transfer.push(&filler[0..avail]).unwrap();
        }

        if knob_signal.signaled() {
            knob_signal.reset();
            let knob = 1;
            if knob != previous_knob {
                previous_knob = knob;
                println!("Hello World - Core 1! {}", knob);
                if knob == 1 {
                    synth.note_on(0, 65);
                    previous_knob = 0;
                }
                if knob == 2 {
                    synth.note_on(0, 66);
                    previous_knob = 0;
                }
            };
        }
    }
}

fn dma_buffer() -> &'static mut [u8; 32000] {
    static mut BUFFER: [u8; 32000] = [0u8; 32000];
    unsafe { &mut BUFFER }
}
