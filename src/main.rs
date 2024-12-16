#![no_std]
#![no_main]

use core::ptr::addr_of_mut;
use embassy_executor::Spawner;
use embassy_sync::channel::{self, Channel};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::cpu_control::{CpuControl, Stack};
use esp_hal::otg_fs::Usb;
use esp_hal::Cpu;
use esp_hal::{
    dma::{Dma, DmaPriority},
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    otg_fs::UsbBus,
    peripherals::Peripherals,
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};

use little_weirdo::synth::effects::overdrive::{KindOfOverdrive};
use usb_device::{
    device::StringDescriptors,
    prelude::{UsbDeviceBuilder, UsbVidPid},
};
use usbd_midi::data::midi::message::control_function::{self, ControlFunction};
use usbd_midi::data::midi::message::Message;
use usbd_midi::{
    data::usb_midi::midi_packet_reader::MidiPacketBufferReader, midi_device::MidiClass,
};

use esp_hal_embassy::Executor;
use little_weirdo::synth::{self, patches::Patches};

//allocations
static mut EP_MEMORY: [u32; 1024] = [0; 1024];
static SHARED: Channel<CriticalSectionRawMutex, usbd_midi::data::midi::message::Message, 20> =
    Channel::new();
static mut APP_CORE_STACK: Stack<64000> = Stack::new();
use static_cell::StaticCell;

#[embassy_executor::task]
pub async fn synth_task() {
    const SAMPLE_RATE: u32 = 44100;
    const AMOUNT_OF_SAMPLES_BUFFERING: usize = 1100;
    const BUFFER_SIZE: usize = 4 * AMOUNT_OF_SAMPLES_BUFFERING;

    esp_println::println!(
        "-:= > Starting LttL Weirdɵ on core {}",
        Cpu::current() as usize
    );

    esp_println::println!("-:= > Starting LttL Weirdɵ @ 16bits / {} Hz", SAMPLE_RATE);

    let mut synth: synth::Synth =
        synth::Synth::new(SAMPLE_RATE as u16, Patches::get_patch(Patches::BassGuitar));
    let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, BUFFER_SIZE);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.channel0;

        let i2s = I2s::new(
            peripherals.I2S0,
            Standard::Philips,
            DataFormat::Data16Channel16,
            SAMPLE_RATE.Hz(),
            dma_channel.configure(false, DmaPriority::Priority0),
            rx_descriptors,
            tx_descriptors,
        );

        let mut i2s_tx = i2s
            .i2s_tx
            .with_bclk(peripherals.GPIO6)
            .with_ws(peripherals.GPIO4)
            .with_dout(peripherals.GPIO5)
            .build();

        esp_println::println!(
            "-:= > Starting LttL Weirdɵ DMA transfer with a {} ms buffer",
            (AMOUNT_OF_SAMPLES_BUFFERING * 1000 / SAMPLE_RATE as usize * 1000) / 1000
        );

        let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            match SHARED.try_receive() {
                Ok(message) => match message {
                    Message::NoteOn(channel, note, velocity) => {
                        synth.note_on(note.into(), velocity.into())
                    }
                    Message::NoteOff(channel, note, velocity) => {
                        synth.note_off(note.into())
                    }
                    Message::ProgramChange(channel, program) => {
                        let program_change:u8 = program.into();
                        match program_change {
                           0x00 => {
                                synth.load_patch(Patches::get_patch(Patches::BassGuitar));
                            }
                            0x01 => {
                                synth.load_patch(Patches::get_patch(Patches::BassGuitarFourth));
                            }
                            0x02 => {
                                synth.load_patch(Patches::get_patch(Patches::BassGuitarMajor));
                            }
                            0x03 => {
                                synth.load_patch(Patches::get_patch(Patches::WeirdScience));
                            }
                            _ => {}
                        }
                    }
                    Message::ControlChange(channel,control_change , value) => {
                        let new_value:u8 = value.into();
                        if control_change == ControlFunction::CHANNEL_VOLUME_7 {
                            synth.change_main_volume( new_value);
                        }
                        if control_change == ControlFunction::SOUND_CONTROLLER_1_70 {
                            synth.change_cutoff( new_value);
                        }
                        if control_change == ControlFunction::SOUND_CONTROLLER_2_71 {
                            synth.change_resonance( new_value);
                        }
                    }
                    _ => {}
                },
                _Error => {}
            };
            match transfer.available() {
                Ok(num_bytes) => {
                    let avail = usize::min(BUFFER_SIZE, num_bytes);
                    for i in 0..(avail / 4) {
                        let output= synth.clock_and_output();
                        let left = output[0].to_be_bytes();
                        let right = output[1].to_be_bytes();
                        // left
                        buffer[i * 4] = left[1];
                        buffer[i * 4 + 1] = left[0];
                        //right
                        buffer[i * 4 + 2] = right[1];
                        buffer[i * 4 + 3] = right[0];
                    }
                    transfer.push(&buffer[0..avail]).unwrap();
                }
                Err(error) => {
                    esp_println::println!("Error: {:?}", error);
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn usb_midi_device_task() {
  

    let usb;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    }
    let usb_bus = UsbBus::new(usb, unsafe { &mut *addr_of_mut!(EP_MEMORY) });
    let mut midi = MidiClass::new(&usb_bus, 1, 1).unwrap();
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x5e4))
        .device_class(0)
        .device_sub_class(0)
        .strings(&[StringDescriptors::default()
            .manufacturer("Little Squeaky Things")
            .product("Little Squeaky Machine")
            .serial_number("00110022")])
        .unwrap()
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut midi]) {
            continue;
        }
        let mut buffer = [0; 64];
        if let Ok(size) = midi.read(&mut buffer) {
            let buffer_reader = MidiPacketBufferReader::new(&buffer, size);
            for packet in buffer_reader.into_iter() {
                if let Ok(packet) = packet {
                    match packet.message {
                        _ => {
                           //esp_println::println!("messsage {:?}", packet.message);
                            SHARED.send(packet.message).await;
                        }
                    }
                }
            }
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::println!("-:=  LttL Squeaky Machine v1.1       =:-");
    esp_println::println!("-:=  Powered by LttL Weirdɵ v0.1     =:-");
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    esp_println::println!(
        "> Set CPU Speed to {:?} MHz\n",
        config.cpu_clock.frequency().to_MHz()
    );
    let peripherals = esp_hal::init(config);

    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    esp_hal_embassy::init([timer1, timer2]);
    //
    let mut cpu_control = CpuControl::new(peripherals.CPU_CTRL);
    let _guard = cpu_control
        .start_app_core(unsafe { &mut *addr_of_mut!(APP_CORE_STACK) }, move || {
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                spawner.spawn(synth_task()).ok();
            });
        })
        .unwrap();
    //
    spawner.spawn(usb_midi_device_task()).ok();
    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
