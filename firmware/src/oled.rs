extern crate alloc;
use alloc::format;
use embassy_time::{Duration, Timer};
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::mono_font::ascii::FONT_7X14_BOLD;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, RoundedRectangle, StyledDrawable};
use embedded_graphics::text::{Alignment, Text};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::*,
    primitives::{Circle, Line, Polyline},
};
use embedded_graphics::{mono_font::MonoTextStyleBuilder, prelude::*};
use embedded_icon::{
    NewIcon,
    mdi::{
        self,
        size24px::{ArrowRightBoldHexagonOutline, Knob, MidiPort, Piano, SineWave},
    },
};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::Peripherals;
use little_weirdo_gui::{DrawItem, EventType};
use little_weirdo_gui::menu::{Menu, MenuIcon};
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};

use embedded_graphics_unicodefonts::MONO_5X8;
use embedded_graphics_unicodefonts::MONO_6X10;

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
            .with_scl(peripherals.GPIO48)
            .into_async();
    }

    let interface = I2CDisplayInterface::new(i2c);
    let mut display: Ssd1306Async<
        ssd1306::prelude::I2CInterface<I2c<'_, esp_hal::Async>>,
        DisplaySize128x64,
        ssd1306::mode::BufferedGraphicsModeAsync<DisplaySize128x64>,
    > = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    let _ = display
        .init_with_addr_mode(ssd1306::command::AddrMode::Horizontal)
        .await;

    let _ = display.flush().await;

    Timer::after(Duration::from_millis(1000)).await;

    let mut menu = Menu::new(EventType::DrumAbout, MenuIcon::Piano);
    menu.add("test1", MenuIcon::Music, EventType::NoMenu);
    menu.add("test2", MenuIcon::Music, EventType::NoMenu);
    menu.add("test3", MenuIcon::Music, EventType::NoMenu);
    

    loop {
        let _ = menu.draw(&mut display);
        let _ = display.flush().await;
        Timer::after(Duration::from_millis(50)).await;
    }
}
