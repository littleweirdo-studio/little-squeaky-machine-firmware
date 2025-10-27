#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use alloc::format;
use embassy_executor::Spawner;
extern crate alloc;
use embassy_time::Duration;
use embassy_time::Timer;
use embassy_usb::Builder;
use embedded_graphics::mono_font::iso_8859_3::FONT_10X20;
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
        ascii::{FONT_5X8, FONT_6X10, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use esp_alloc as _;

use embedded_graphics::{image::Image, prelude::*};
use esp_alloc as _;
use esp_backtrace as _;
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
use esp_hal::otg_fs::asynch::Driver;
use esp_hal::otg_fs::Usb;
use esp_hal::peripheral;
use esp_hal::peripheral::Peripheral;
use esp_hal::peripherals::ADC1;
use esp_hal::psram::{self, PsramSize};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::{timg::TimerGroup, AnyTimer};
use esp_println::println;
use esp_storage::FlashStorage;

use embedded_storage::{ReadStorage, Storage};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // init CPU
    let mut config = esp_hal::Config::default();
    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    esp_hal_embassy::init([timer1, timer2]);
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);

    let mut storage = FlashStorage::new();
    println!("Flash size = {} MB", storage.capacity() / 1_000_000);
    println!("Writing to flash");
    let _ = storage.write(0x00220000, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav0.raw"));
    let _ = storage.write(0x00220000 + 1 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav1.raw"));
    let _ = storage.write(0x00220000 + 2 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav2.raw"));
    let _ = storage.write(0x00220000 + 3 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav3.raw"));
    let _ = storage.write(0x00220000 + 4 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav4.raw"));
    let _ = storage.write(0x00220000 + 5 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav5.raw"));
    let _ = storage.write(0x00220000 + 6 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav6.raw"));
    let _ = storage.write(0x00220000 + 7 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav7.raw"));
    let _ = storage.write(0x00220000 + 8 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav8.raw"));
    let _ = storage.write(0x00220000 + 9 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/wav9.raw"));
    let _ = storage.write(0x00220000 + 10 * 1200, include_bytes!("../src/soundbank/soundbank_pure_elektro/src/sample.raw"));
    println!("Writing to flash done");
}
