#![no_std]
#![no_main]


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

         let character_style = MonoTextStyle::new(&FONT_10X20, BinaryColor::On);
    
        let analog_pin = peripherals.GPIO8;
    let mut adc_config = AdcConfig::default();
    

    let mut pot_in0 = adc_config.enable_pin_with_cal::<GpioPin<8>,  AdcCalBasic<ADC1>>(analog_pin, Attenuation::Attenuation11dB);
    
    let mut adc1 = Adc::new(peripherals.ADC1, adc_config);   
    loop {
        Timer::after(Duration::from_millis(100)).await;
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
        Text::with_alignment(
            text.as_str(),
            Point::new(0, 40),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display).unwrap();
        display.flush();

    }    
   
}




