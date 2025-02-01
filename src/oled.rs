extern crate alloc;
use alloc::format;
use embedded_graphics::image::{Image, ImageRaw};
use fugit::RateExtU32;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::{FONT_7X14_BOLD};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::{Alignment, Text};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcCalBasic, AdcConfig, Attenuation};
use esp_hal::gpio::GpioPin;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::{Peripherals, ADC1};
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use usbd_midi::message::control_function::ControlFunction;
use usbd_midi::message::{Message, U7};

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
};
use ssd1306::mode::DisplayConfig;
use crate::MIDI_MESSAGE_CHANNEL;

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
    let mut adc1;
    let mut pot_in0;
    let i2c;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let mut config = Config::default();
        config.frequency =  400u32.kHz();
        
        i2c = I2c::new(peripherals.I2C0, config).unwrap()
        .with_sda(peripherals.GPIO41)
        .with_scl(peripherals.GPIO42);

        let analog_pin = peripherals.GPIO8;
        let mut adc_config = AdcConfig::default();

        pot_in0 = adc_config.enable_pin_with_cal::<GpioPin<8>, AdcCalBasic<ADC1>>(
            analog_pin,
            Attenuation::_11dB,
        );

        adc1 = Adc::new(peripherals.ADC1, adc_config);
    }
   
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init();
    display.clear_buffer();
    display.flush().unwrap();

    let character_style = MonoTextStyleBuilder::new().font(&FONT_7X14_BOLD).background_color(BinaryColor::On).text_color(BinaryColor::Off).build();
    let image_raw = ImageRaw::<BinaryColor>::new(DATA,12);
    Image::new(&image_raw, Point::new(0, 0)).draw(&mut display).unwrap();
    Text::with_alignment(
        "LttL",
        Point::new(0, 24),
        character_style,
        Alignment::Left,
    )
    .draw(&mut display)
    .unwrap();
    let _ = display.flush();
    Timer::after(Duration::from_millis(1000)).await;
  
    loop {
        let pin_value: u16 = adc1.read_blocking(&mut pot_in0);
        let text = format!("value {:?}", pin_value);
        let test = U7::try_from((pin_value / 40) as u8);
        match test {
            Ok(value) => {
                MIDI_MESSAGE_CHANNEL
                    .send(Message::ControlChange(
                        usbd_midi::message::Channel::Channel1,
                        ControlFunction::SOUND_CONTROLLER_1_70,
                        value,
                    ))
                    .await;
            }
            Err(_) => {}
        }
        display.clear_buffer();
        Image::new(&image_raw, Point::new(0, 0)).draw(&mut display).unwrap();
        Text::with_alignment(
            "LttL Weirdo",
            Point::new(0, 24),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        Text::with_alignment(
            &text,
            Point::new(0, 48),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        let _ = display.flush();
        Timer::after(Duration::from_millis(100)).await;
    }
}