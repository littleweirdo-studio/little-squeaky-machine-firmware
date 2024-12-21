#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

extern crate alloc;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::{Alignment, Text};
use esp_alloc as _;
use alloc::format;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use esp_hal::analog::adc::{Adc, AdcCalBasic, AdcConfig, Attenuation};
use esp_hal::gpio::GpioPin;
use esp_hal::peripherals::ADC1;
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use usbd_midi::data::byte::u7::U7;
use core::ptr::addr_of_mut;
use embassy_executor::Spawner;
use embassy_sync::channel::{self, Channel};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::cpu_control::{CpuControl, Stack};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::otg_fs::Usb;
use esp_hal::{psram, Cpu};
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
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD, FONT_5X8},
        MonoTextStyleBuilder,
    },
    prelude::*,
};


//allocations
static mut EP_MEMORY: [u32; 1024] = [0; 1024];
static SHARED: Channel<CriticalSectionRawMutex, usbd_midi::data::midi::message::Message, 50> =
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
async fn oled_task() {
    let mut adc1;
    let mut pot_in0;
    let mut i2c;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let mut config = Config::default();
       i2c = I2c::new(
             peripherals.I2C0,
             config,
        ).with_sda(peripherals.GPIO17)
         .with_scl(peripherals.GPIO18);

         let analog_pin = peripherals.GPIO8;
         let mut adc_config = AdcConfig::default();
         
     
        pot_in0 = adc_config.enable_pin_with_cal::<GpioPin<8>,  AdcCalBasic<ADC1>>(analog_pin, Attenuation::Attenuation11dB);
         
         adc1 = Adc::new(peripherals.ADC1, adc_config);   
 
    }

    let character_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
 
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init();
    display.clear_buffer();
    let pin_value:u16 = adc1.read_blocking(&mut pot_in0);
    let text = format!("value {:?}", pin_value);
    Text::with_alignment(
        "LttL Weirdo",
        Point::new(0, 20),
        character_style,
        Alignment::Left,
    )
    .draw(&mut display).unwrap();
    display.flush();

  
    loop {
        display.clear_buffer();
        let pin_value:u16 = adc1.read_blocking(&mut pot_in0);
        let text = format!("value {:?}", pin_value);
        let test = U7::try_from((pin_value/40) as u8);
        match test {
            Ok(value) => {
                SHARED.send(Message::ControlChange(usbd_midi::data::midi::channel::Channel::Channel1, ControlFunction::SOUND_CONTROLLER_1_70, value)).await;
            },
            Err(_) => {},
        }

        Text::with_alignment(
            "LttL Weirdo",
            Point::new(0, 20),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display).unwrap();
         Text::with_alignment(
            text.as_str(),
            Point::new(0, 40),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display).unwrap();
        display.flush();
        Timer::after(Duration::from_millis(100)).await;
  
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
            Timer::after(Duration::from_micros(10)).await;
    }
        
     
    }
}



#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    esp_println::println!("-:=  LttL Squeaky Machine v1.1       =:-");
    esp_println::println!("-:=  Powered by LttL Weirdɵ v0.1     =:-");
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    esp_println::println!(
        "> Set CPU Speed to {:?} MHz\n",
        config.cpu_clock.frequency().to_MHz()
    );
    let peripherals = esp_hal::init(config);
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);
   
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer_group_1 = TimerGroup::new(peripherals.TIMG1);
    
    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    let timer3: AnyTimer = timer_group_1.timer0.into();
    
    esp_hal_embassy::init([timer1, timer2, timer3]);
    //
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
    spawner.must_spawn(usb_midi_device_task());
    spawner.must_spawn(oled_task());
    
   loop {
    Timer::after(Duration::from_millis(1000)).await;
    }
   
}
