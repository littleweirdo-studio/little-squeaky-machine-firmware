#![no_std]
#![no_main]

use core::ptr::addr_of_mut;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::cpu_control::{CpuControl, Stack};
use esp_hal::touch::{Touch, TouchPad};
use esp_hal::Cpu;
use esp_hal::{
    dma::{Dma, DmaPriority},
    dma_circular_buffers,
    i2s::master::{DataFormat, I2s, Standard},
    peripherals::Peripherals,
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};

use esp_hal_embassy::Executor;
use little_weirdo::synth::{self, patches::Patches};

mod midi;
use midi::{Channel, KeyEvent, MidiMessage};
static SHARED: Signal<CriticalSectionRawMutex, MidiMessage> = Signal::new();

static mut APP_CORE_STACK: Stack<32000> = Stack::new();
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

    esp_println::println!(
        "-:= > Starting LttL Weirdɵ @ 16bits / {} Hz",
        SAMPLE_RATE
    );

    let mut synth: synth::Synth =
        synth::Synth::new(SAMPLE_RATE as u16, Patches::get_patch(Patches::EletricPiano));
    let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, BUFFER_SIZE);

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.i2s0channel;

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
            .with_bclk(peripherals.GPIO27)
            .with_ws(peripherals.GPIO33)
            .with_dout(peripherals.GPIO32)
            .build();

            esp_println::println!(
                "-:= > Starting LttL Weirdɵ DMA transfer with a {} ms buffer",
                 (AMOUNT_OF_SAMPLES_BUFFERING * 1000 / SAMPLE_RATE as usize * 1000) / 1000
            );
        let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            match  SHARED.try_take() {
                Some(message) => match message {
                    MidiMessage::NoteOn(channel, note) => {
                        synth.note_on(0x00, note.key, note.value)
                    }
                    MidiMessage::NoteOff(channel, note) => {
                        synth.note_off(0x00, note.key)
                    }
                    MidiMessage::ProgramChange(channel, program) => {
                  
                        match program {
                            1 => {
                                synth.filter.filter_on = true;
                                synth.overdrive_active = true;
                            }
                            2 => {
                                synth.filter.filter_on = false;
                                synth.overdrive_active = false;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                None => {},
            };
            match transfer.available() {
                Ok(num_bytes) => {
                    let avail = usize::min(BUFFER_SIZE, num_bytes);
                    for i in 0..(avail / 4) {
                        let output: i16 = synth.clock_and_output();
                        let bytes = output.to_be_bytes();
                        // left
                        buffer[i * 4] = bytes[1];
                        buffer[i * 4 + 1] = bytes[0];
                        //right
                        buffer[i * 4 + 2] = bytes[1];
                        buffer[i * 4 + 3] = bytes[0];
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
async fn intro_text() {
    esp_println::println!("-:=  LttL Squeaky Machine v1.1       =:-");
    esp_println::println!("-:=  Powered by LttL Weirdɵ v0.1     =:-");
    esp_println::println!("");
    esp_println::println!(
        "-:= > Starting LttL Squeaky Machine on core {}",
        Cpu::current() as usize
    );
    let mut active_note: bool = false;
    let mut active_change: bool = false;
    let mut touchpad1;
   let mut touchpad2;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let touch_pin0 = peripherals.GPIO13;
        let touch = Touch::continuous_mode(peripherals.TOUCH, None);
        touchpad1 = TouchPad::new(touch_pin0, &touch);
        let touch_pin2 = peripherals.GPIO12;
       touchpad2 = TouchPad::new(touch_pin2, &touch);
    }
    let mut key = 24;
    loop {
        Timer::after(Duration::from_millis(5)).await;
        let touch_val_1 = touchpad1.read();
        if touch_val_1 < 150 && !active_note {
            SHARED.signal(MidiMessage::NoteOn(Channel::from(0),  KeyEvent { key: key, value: 127 }));
            active_note = true;
        }
        if touch_val_1 > 250 && active_note {
            SHARED.signal(MidiMessage::NoteOff(Channel::from(0), KeyEvent { key: key, value: 0  }));
            active_note = false;
            key = key + 1;
            if key > 37 { key = 24 };
        }
       let _touch_val_2 = touchpad2.read();
        if _touch_val_2 < 150 && !active_change {
          SHARED.signal(MidiMessage::ProgramChange(Channel::from(0), 1));
          active_change = true;
        }
        if _touch_val_2 > 250 && active_change {
            active_change = false;
            SHARED.signal(MidiMessage::ProgramChange(Channel::from(0), 2));
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    esp_println::println!(
        "> Set ESP32 Speed to {:?} MHz\n\n",
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
    spawner.spawn(intro_text()).ok();
    loop {
        Timer::after(Duration::from_millis(1000)).await;
    }
}
