extern crate alloc;
use embassy_time::{Duration, Timer};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::peripherals::Peripherals;
use esp_println::println;
use little_weirdo_gui::about::About;
use little_weirdo_gui::menu::{Menu, MenuIcon};
use little_weirdo_gui::{ActiveItem, DrawItem, EventType, Navigatable, SHARED};
use midi_msg::Channel;
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};

use crate::MIDI_MESSAGE_CHANNEL;

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

    let mut main_menu = Menu::new(EventType::NoMenu, MenuIcon::Sine);
    main_menu.add("DRUMS", MenuIcon::Music, EventType::DrumMenu);
    main_menu.add("SEQUENCER", MenuIcon::Piano, EventType::SeqMenu);
    main_menu.add("SYNTH", MenuIcon::Sine, EventType::NoMenu);
    main_menu.add("MIDI", MenuIcon::Midi, EventType::NoMenu);
    main_menu.add("SETTINGS", MenuIcon::Tune, EventType::NoMenu);
    main_menu.add("STATUS", MenuIcon::Status, EventType::NoMenu);
    main_menu.add("ABOUT", MenuIcon::Information, EventType::LttLMenuAbout);

   let mut drum_menu = Menu::new(EventType::MainMenu, MenuIcon::Music);
    drum_menu.add("PATCHES", MenuIcon::Patch, EventType::DrumPatchMenu);
    drum_menu.add("SETTINGS", MenuIcon::Tune, EventType::NoMenu);
    drum_menu.add("ABOUT", MenuIcon::Information, EventType::DrumAbout);

    let mut seq_menu = Menu::new(EventType::MainMenu, MenuIcon::Piano);
    seq_menu.add("PATCHES", MenuIcon::Patch, EventType::DrumPatchMenu);
    seq_menu.add("SETTINGS", MenuIcon::Tune, EventType::NoMenu);
    seq_menu.add("ABOUT", MenuIcon::Information, EventType::DrumAbout);

    let mut drum_patch_menu = Menu::new(EventType::DrumMenu, MenuIcon::Patch);
    drum_patch_menu.add("TR 808", MenuIcon::Drum, EventType::NoMenu);
    drum_patch_menu.add("TR 909", MenuIcon::Drum, EventType::NoMenu);
    drum_patch_menu.add("WEIRD", MenuIcon::Drum, EventType::NoMenu);

    let mut about_drums_menu = About::new(
        "LTTL Weirdo Drums is a collection of drum sounds for the LTTL Weirdo synthesizer.",
        EventType::DrumMenu,
    );

    let mut about_lttl_menu = About::new(
        "LttL Hi SQυεΛκY Things Machine v1.0 ♫ §QυεΛΛΛΛκκ Cαη ħαρρεη α⊥ αηψ тıмε! ©'26 HST∞LAB ♥♪♥",
        EventType::MainMenu,
    );
    

    let mut active_item = ActiveItem::Menu(&mut main_menu);

    
    loop {
        let _ = display.clear(BinaryColor::Off);
        let _ = active_item.draw(&mut display);
        let _ = display.flush().await;
        match SHARED.receive().await {
                 EventType::DrumMenu => active_item = ActiveItem::Menu(&mut drum_menu),
                EventType::NoMenu => {}
                EventType::MainMenu => active_item = ActiveItem::Menu(&mut main_menu),
                EventType::DrumPatchMenu => active_item = ActiveItem::Menu(&mut drum_patch_menu),
                EventType::DrumAbout => active_item = ActiveItem::About(&mut about_drums_menu),
                EventType::SeqMenu => active_item = ActiveItem::Menu(&mut seq_menu),
                EventType::LttLMenuAbout => active_item = ActiveItem::About(&mut about_lttl_menu),
                EventType::KeypadUp => active_item.next(),
                EventType::KeypadDown => active_item.prev(),
                EventType::KeypadLeft => active_item.back(),
                EventType::KeypadRight => active_item.select(),
            _ => {}
        }
    }
}
