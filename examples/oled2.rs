#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use alloc::format;
use embassy_executor::Spawner;
extern crate alloc;
use embedded_graphics::mono_font::iso_8859_3::FONT_10X20;
use esp_alloc as _;
use embassy_time::Duration;
use embassy_time::Timer;
use embedded_graphics::mono_font::mapping::StrGlyphMapping;
use embedded_graphics::mono_font::mapping::ASCII;
use embedded_graphics::mono_font::MonoFont;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Gray8;
use embedded_graphics::primitives::Polyline;
use embedded_graphics::primitives::PrimitiveStyle;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::primitives::StrokeAlignment;
use embedded_graphics::text::Baseline;
use embedded_graphics::text::TextStyle;
use embedded_graphics::text::TextStyleBuilder;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD, FONT_5X8},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};

use esp_backtrace as _;
use esp_alloc as _;
use esp_hal::analog::adc;
use esp_hal::analog::adc::Adc;
use esp_hal::analog::adc::AdcCalBasic;
use esp_hal::analog::adc::AdcCalLine;
use esp_hal::analog::adc::AdcConfig;
use esp_hal::analog::adc::Attenuation;
use esp_hal::gpio::AnalogPin;
use esp_hal::gpio::GpioPin;
use esp_hal::gpio::Input;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::psram::{self, PsramSize};
use esp_hal::peripheral;
use esp_hal::peripheral::Peripheral;
use esp_hal::peripherals::ADC1;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::{
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};
use esp_println::println;
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};


#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // init CPU
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    esp_hal_embassy::init([timer1, timer2]);
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);
   

    let mut config = Config::default();
    let mut i2c = I2c::new(
         peripherals.I2C0,
         config,
    ).with_sda(peripherals.GPIO17)
     .with_scl(peripherals.GPIO18);
 
  
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init();
    display.clear_buffer();
    display.flush();

    let character_style1 = MonoTextStyleBuilder::new().font(&FONT_10X20).background_color(BinaryColor::Off).text_color(BinaryColor::On).build();

         let character_style2 = MonoTextStyleBuilder::new().font(&FONT_10X20).background_color(BinaryColor::On).text_color(BinaryColor::Off).build();

        let analog_pin = peripherals.GPIO8;
    let mut adc_config = AdcConfig::default();
    

    let mut pot_in0 = adc_config.enable_pin_with_cal::<GpioPin<8>,  AdcCalBasic<ADC1>>(analog_pin, Attenuation::Attenuation11dB);
    
    let mut adc1 = Adc::new(peripherals.ADC1, adc_config);   
    loop {
        display.clear_buffer();
        let pin_value:u16 = adc1.read_blocking(&mut pot_in0) / 1000;
        let mut str: [&str;3] = ["";3];
        match pin_value {
            0 => {
                str[0] = "> menu 0";
                str[1] = "  menu 1";
                str[2] = "  menu 2";

            },
            1 => {
                str[0] = "> menu 1";
                str[1] = "  menu 2";
                str[2] = "  menu 3";

            },
            2 => {
                str[0] = "> menu 2";
                str[1] = "  menu 3";
                str[2] = "  menu 4";

            },
            3 => {
                str[0] = "> menu 3";
                str[1] = "  menu 4";
                str[2] = "";

            },
            4 => {
                str[0] = "> menu 4";
                str[1] = "";
                str[2] = "";

            },
            _ => {
                str[0] = "  menu x"
            },
        }
        let mut count = 0;
        for text in str {
            if count == 0 {
                Text::with_alignment(
                    text,
                    Point::new(20, 18 + count * 20),
                    character_style2,
                    Alignment::Left,
                )
                .draw(&mut display).unwrap();
            } else {
                Text::with_alignment(
                    text,
                    Point::new(20, 18 + count * 20),
                    character_style1,
                    Alignment::Left,
                )
                .draw(&mut display).unwrap();
           // }
            }
            count = count + 1;
        }
        display.flush();
          
    }    
   
}


