#![no_std]
#![no_main]
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
use esp32_hal::entry;
use esp32_hal::{
    clock::ClockControl, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*,
    timer::TimerGroup,
};
use esp_backtrace as _;
use esp_println::println;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::mono_font::mapping::{GlyphMapping, StrGlyphMapping};

pub const PICO_FONT_4X6: MonoFont = MonoFont {
    image: embedded_graphics::image::ImageRaw::new(include_bytes!("picofont_4x6.raw"), 128),
    glyph_mapping: &StrGlyphMapping::new("\0\u{20}\u{7f}", 0),
    character_size: embedded_graphics::geometry::Size::new(8, 6),
    character_spacing: 0,
    baseline: 6,
    underline: embedded_graphics::mono_font::DecorationDimensions::new(4 + 2, 1),
    strikethrough: embedded_graphics::mono_font::DecorationDimensions::new(6 / 2, 1),
};
 
Ò
    println!(" _01sss23456");
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Create a new peripheral object with the described wiring
    // and standard I2C clock speed
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio21,
        io.pins.gpio22,
        100u32.kHz(),
        &clocks,
    );
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    /*let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    // A "heartbeat" shaped polyline (128x64)
    let points: [Point; 10] = [
        Point::new(10, 0),
        Point::new(20, 10),
        Point::new(30, 0),
        Point::new(40, 15),
        Point::new(50, 16),
        Point::new(60, 30),
        Point::new(70, 16),
        Point::new(80, 40),
        Point::new(90, 16),
        Point::new(100, 50),
    ];

    Polyline::new(&points)
        .into_styled(thin_stroke)
        .draw(&mut display)
        .unwrap();
*/



let raw_image: embedded_graphics::image::ImageRaw<'_, BinaryColor> = embedded_graphics::image::ImageRaw::<BinaryColor>::new(include_bytes!("frame.raw"), 128);
let image = embedded_graphics::image::Image::new(&raw_image, Point::zero());
image.draw(&mut display).unwrap();

/* 
    let character_style = MonoTextStyle::new(&PICO_FONT_4X6, BinaryColor::On);
    let text = "-: = LTTL WEIRDO!";
    Text::with_alignment(
        text,
        Point::new(0, 6),
        character_style,
        Alignment::Left,
    )
    .draw(&mut display).unwrap();
*/

    display.flush().unwrap();
    loop {}
}
