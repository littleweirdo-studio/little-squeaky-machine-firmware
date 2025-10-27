extern crate alloc;
use alloc::format;
use embassy_time::{Duration, Timer};
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::ascii::FONT_7X14_BOLD;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::{mono_font::MonoTextStyleBuilder, prelude::*};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::Peripherals;
use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};

use crate::potmeter::{
    POT1_MESSAGE_CHANNEL, POT2_MESSAGE_CHANNEL, POT3_MESSAGE_CHANNEL, POT4_MESSAGE_CHANNEL,
    USB_POWER_MESSAGE_CHANNEL,
};

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
        let config = Config::default();
    

        i2c = I2c::new(peripherals.I2C0, config)
            .unwrap()
            .with_sda(peripherals.GPIO47)
            .with_scl(peripherals.GPIO48);
    }

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    let _ = display.init();
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

    let mut receiver_pot1 = POT1_MESSAGE_CHANNEL.receiver().unwrap();
    let mut receiver_pot2 = POT2_MESSAGE_CHANNEL.receiver().unwrap();
    let mut receiver_pot3 = POT3_MESSAGE_CHANNEL.receiver().unwrap();
    let mut receiver_pot4 = POT4_MESSAGE_CHANNEL.receiver().unwrap();
    let mut receiver_usb = USB_POWER_MESSAGE_CHANNEL.receiver().unwrap();

    loop {
        let value_pot_1 = receiver_pot1.changed().await;
        let value_pot_2 = receiver_pot2.changed().await;
        let value_pot_3 = receiver_pot3.changed().await;
        let value_pot_4 = receiver_pot4.changed().await;
        let usb_power = receiver_usb.changed().await;

        display.clear_buffer();

        Text::with_alignment(
            format!("p={}", usb_power).as_str(),
            Point::new(70, 10),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();

        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(0, 2), Size::new(value_pot_1 as u32 / 4, 10)),
            Size {
                width: 2,
                height: 2,
            },
        )
        .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
        .draw(&mut display)
        .unwrap();
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(0, 19), Size::new(value_pot_2 as u32 / 4, 10)),
            Size {
                width: 2,
                height: 2,
            },
        )
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
        .draw(&mut display)
        .unwrap();
        Rectangle::new(Point::new(0, 34), Size::new(value_pot_3 as u32 / 4, 10))
            .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
            .draw(&mut display)
            .unwrap();
        Rectangle::new(Point::new(0, 51), Size::new(value_pot_4 as u32 / 4, 10))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 2))
            .draw(&mut display)
            .unwrap();

        Rectangle::new(Point::new(0, 0), Size::new(20, 64))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut display)
            .unwrap();

        Rectangle::new(Point::new(0, 0), Size::new(64, 64))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 4))
            .draw(&mut display)
            .unwrap();

        /*
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
        .unwrap();*/
        /*  Text::with_alignment(
                format!("pot 1 = {}", value_pot_1).as_str(),
                Point::new(0, 44),
                character_style,
                Alignment::Left,
            )
            .draw(&mut display)
            .unwrap();
        Text::with_alignment(
            format!("pot 2= {}", value_pot_2).as_str(),
            Point::new(0, 64),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display) */
        //.unwrap();

        let _ = display.flush();
        Timer::after(Duration::from_millis(50)).await;
    }
}
