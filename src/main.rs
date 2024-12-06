#![no_std]
#![no_main]

use core::ptr::addr_of_mut;

use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    signal::Signal,
};
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
static SHARED: Signal<CriticalSectionRawMutex, bool> = Signal::new();

static mut APP_CORE_STACK: Stack<32000> = Stack::new();
use static_cell::StaticCell;

#[embassy_executor::task]
pub async fn synth_task() {
    const SAMPLE_RATE: u32 = 22050;
    const BUFFER_SIZE: usize = 4 * 1100;

    esp_println::println!(
        "Starting synth_task() on core {}",
        Cpu::current() as usize
    );

    let mut synth: synth::Synth = synth::Synth::new(
        SAMPLE_RATE as u16,
        Patches::get_patch(Patches::BassGuitar),
    );
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

        let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            Timer::after(Duration::from_nanos(1)).await;
            let signal = SHARED.try_take();
            match signal {
                Some(activate) => {
                    if activate {
                        synth.note_on(0, 40, 100);
                    } else {
                        synth.note_off(0, 40);
                    }
                }
                None => {}
            }
            match transfer.available() {
                Ok(num_bytes) => {
                    let avail = usize::min(BUFFER_SIZE, num_bytes);
                    for i in 0..(avail / 4) {
                        let output: i16 = synth.clock_and_output();
                        let bytes = output.to_be_bytes();
                        buffer[i * 4] = bytes[1];
                        buffer[i * 4 + 1] = bytes[0];
                        buffer[i * 4 + 2] = bytes[1];
                        buffer[i * 4 + 3] = bytes[0];
                    }
                    transfer.push(&buffer[0..avail]).unwrap();
                }
                Err(error) => {
                    // Handle error (e.g., log it or take corrective action)
                    esp_println::println!("Error: {:?}", error);
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn intro_text() {

    esp_println::println!("-:=  LttL Squeaky Machine v1.1       =:-");
    esp_println::println!("-:=  Powered by LttL Weirdo v0.1     =:-");
    esp_println::println!(
        "Starting intro_text() on core {}",
        Cpu::current() as usize
    );
    let mut active: bool = false;

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
    

        let touch_pin0 = peripherals.GPIO13;
        let touch = Touch::continuous_mode(peripherals.TOUCH, None);
        let mut touchpad = TouchPad::new(touch_pin0, &touch);

        loop {
            Timer::after(Duration::from_millis(10)).await;
            let touch_val = touchpad.read();
            if touch_val < 100 && !active {
                SHARED.signal(true);
                active = true
            }

            if touch_val > 200 && active {
                SHARED.signal(false);
                active = false;
            }
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    //
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
