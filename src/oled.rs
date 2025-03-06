extern crate alloc;
use embassy_time::{Duration, Timer};
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::ascii::FONT_7X14_BOLD;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::{Alignment, Text};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::{Peripherals};
use fugit::RateExtU32;
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use embedded_graphics::{mono_font::MonoTextStyleBuilder, prelude::*};
use ssd1306::mode::DisplayConfig;

#[rustfmt::skip]
const DATA: &[u8] = &[
    0b11101111, 0b0101_0000,
    0b10001000, 0b0101_0000,
    0b11101011, 0b0101_0000,
    0b10001001, 0b0101_0000,
    0b11101111, 0b0101_0000,
];

#[embassy_executor::task]
pub async fn oled_task() {
    let i2c;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let mut config = Config::default();
        config.frequency = 400u32.kHz();

        i2c = I2c::new(peripherals.I2C0, config)
            .unwrap()
            .with_sda(peripherals.GPIO47)
            .with_scl(peripherals.GPIO48);
    }

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init();
    display.clear_buffer();
    display.flush().unwrap();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_7X14_BOLD)
        .background_color(BinaryColor::On)
        .text_color(BinaryColor::Off)
        .build();
    let image_raw = ImageRaw::<BinaryColor>::new(DATA, 12);
    Image::new(&image_raw, Point::new(0, 0))
        .draw(&mut display)
        .unwrap();
    Text::with_alignment("LttL", Point::new(0, 24), character_style, Alignment::Left)
        .draw(&mut display)
        .unwrap();
    let _ = display.flush();
    Timer::after(Duration::from_millis(1000)).await;

    loop {
        display.clear_buffer();
        Image::new(&image_raw, Point::new(0, 0))
            .draw(&mut display)
            .unwrap();
        Text::with_alignment(
            "LttL Weirdo",
            Point::new(0, 24),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();

        let _ = display.flush();
        Timer::after(Duration::from_millis(50)).await;
    }
}
