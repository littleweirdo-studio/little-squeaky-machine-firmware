extern crate alloc;
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::Peripherals;
use little_weirdo_gui::{DrawItem, EventType};
use little_weirdo_gui::menu::{Menu, MenuIcon};
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};


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

    let mut main_menu =Menu::new(EventType::NoMenu, MenuIcon::Sine);
    main_menu.add("DRUMS", MenuIcon::Music, EventType::DrumMenu);
    main_menu.add("SEQUENCER", MenuIcon::Piano, EventType::SeqMenu);
    main_menu.add("SYNTH", MenuIcon::Sine, EventType::NoMenu);
    main_menu.add("MIDI", MenuIcon::Midi, EventType::NoMenu);
    main_menu.add("SETTINGS", MenuIcon::Tune, EventType::NoMenu);
    main_menu.add("STATUS", MenuIcon::Status, EventType::NoMenu);
    main_menu.add("ABOUT", MenuIcon::Information, EventType::LttLMenuAbout);

    loop {
        let _ = main_menu.draw(&mut display);
        let _ = display.flush().await;
        Timer::after(Duration::from_millis(50)).await;
    }
}
