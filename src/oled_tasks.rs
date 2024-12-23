extern crate alloc;
use ssd1306::command::AddrMode;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::text::{Alignment, Text};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::analog::adc::{Adc, AdcCalBasic, AdcConfig, Attenuation};
use esp_hal::gpio::GpioPin;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::ADC1;
use esp_hal::{
    peripherals::Peripherals,
    prelude::*,
};
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface,  Ssd1306Async};
use usbd_midi::data::byte::u7::U7;
use usbd_midi::data::midi::message::control_function::ControlFunction;
use usbd_midi::data::midi::message::Message;

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::*,
};

use crate::MIDI_MESSAGE_CHANNEL;


#[embassy_executor::task]
pub async fn oled_task() {
    let mut adc1;
    let mut pot_in0;
    let i2c;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let mut config = Config::default();
        config.frequency =  400u32.kHz();
        
        i2c = I2c::new(peripherals.I2C0, config)
            .with_sda(peripherals.GPIO17)
            .with_scl(peripherals.GPIO18)
            .into_async();

        let analog_pin = peripherals.GPIO8;
        let mut adc_config = AdcConfig::default();

        pot_in0 = adc_config.enable_pin_with_cal::<GpioPin<8>, AdcCalBasic<ADC1>>(
            analog_pin,
            Attenuation::Attenuation11dB,
        );

        adc1 = Adc::new(peripherals.ADC1, adc_config);
    }
   
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();
    display.init_with_addr_mode(AddrMode::Page).await.unwrap();
    display.clear_buffer();
    display.clear(BinaryColor::Off).unwrap();
    display.flush().await.unwrap();
    let character_style = MonoTextStyleBuilder::new().font(&FONT_10X20).background_color(BinaryColor::On).text_color(BinaryColor::Off).build();

    //Image::new(&image_raw, Point::new(20, 20)).draw(&mut display).unwrap();
    Text::with_baseline(
        "LttL",
        Point::new(50, 50),
        character_style,
       embedded_graphics::text::Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    let _ = display.flush().await;
    Timer::after(Duration::from_millis(1000)).await;
  
    loop {
        let pin_value: u16 = adc1.read_blocking(&mut pot_in0);
        //let text = format!("value {:?}", pin_value);
        let test = U7::try_from((pin_value / 40) as u8);
        match test {
            Ok(value) => {
                MIDI_MESSAGE_CHANNEL
                    .send(Message::ControlChange(
                        usbd_midi::data::midi::channel::Channel::Channel1,
                        ControlFunction::SOUND_CONTROLLER_1_70,
                        value,
                    ))
                    .await;
            }
            Err(_) => {}
        }
        display.clear_buffer();
        Text::with_alignment(
            "LttL Weirdo",
            Point::new(0, 24),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        Text::with_alignment(
            "test",
            Point::new(0, 48),
            character_style,
            Alignment::Left,
        )
        .draw(&mut display)
        .unwrap();
        let _ = display.flush().await;
        Timer::after(Duration::from_millis(100)).await;
    }
}