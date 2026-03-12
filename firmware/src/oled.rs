extern crate alloc;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
use esp_alloc as _;
use esp_backtrace as _;
use little_weirdo_gui::about::About;
use little_weirdo_gui::menu::{self, MenuIcon};
use little_weirdo_gui::{ActiveItem, DrawItem, EventType, NAVIGATION_SIGNAL, Navigatable};
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};

use crate::I2c1Bus;

#[embassy_executor::task]
pub async fn oled_task(i2c_bus: &'static I2c1Bus ) {
    let i2c_dev = I2cDevice::new(i2c_bus);

    let interface = I2CDisplayInterface::new(i2c_dev);
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    let _ = display
        .init_with_addr_mode(ssd1306::command::AddrMode::Horizontal)
        .await;

    let _ = display.flush().await;

    Timer::after(Duration::from_millis(1000)).await;

let mut main_menu = menu::Menu::new(EventType::NoMenu, EventType::NoActive, MenuIcon::Sine);
    main_menu.add("DRUMS", MenuIcon::Music, EventType::DrumMenu,EventType::NoActive);
    main_menu.add("SEQUENCER", MenuIcon::Piano, EventType::SeqMenu, EventType::NoActive);
    main_menu.add("SYNTH", MenuIcon::Sine, EventType::SynthMenu, EventType::NoActive);
    main_menu.add("MIDI", MenuIcon::Midi, EventType::NoMenu, EventType::NoActive);
    main_menu.add("SETTINGS", MenuIcon::Tune, EventType::NoMenu, EventType::NoActive);
    main_menu.add("STATUS", MenuIcon::Status, EventType::NoMenu, EventType::NoActive);
    main_menu.add("ABOUT", MenuIcon::Information, EventType::LttLMenuAbout, EventType::NoActive);


    let mut synth_menu =  menu::Menu::new(EventType::MainMenu, EventType::SynthModeActive,MenuIcon::Sine);
    synth_menu.add("PATCHES", MenuIcon::Patch, EventType::SynthPatchMenu, EventType::NoActive);
   
    let mut synth_patch_menu = menu::Menu::new(EventType::SynthMenu, EventType::NoActive,MenuIcon::Patch);
    synth_patch_menu.add("BASS", MenuIcon::Sine, EventType::NoMenu, EventType::PatchSelect);
    synth_patch_menu.add("E-BASS", MenuIcon::Sine, EventType::NoMenu, EventType::PatchSelect);
    synth_patch_menu.add("WEIRD", MenuIcon::Sine, EventType::NoMenu, EventType::PatchSelect);
 synth_patch_menu.add("X", MenuIcon::Sine, EventType::NoMenu, EventType::PatchSelect);
    synth_patch_menu.add("Y", MenuIcon::Sine, EventType::NoMenu, EventType::PatchSelect);


    let mut drum_menu = menu::Menu::new(EventType::MainMenu, EventType::DrumModeActive,MenuIcon::Music);
    drum_menu.add("PATCHES", MenuIcon::Patch, EventType::DrumPatchMenu, EventType::NoActive);
    drum_menu.add("SETTINGS", MenuIcon::Tune, EventType::NoMenu, EventType::NoActive);
    drum_menu.add("ABOUT", MenuIcon::Information, EventType::DrumAbout, EventType::NoActive);
    
    let mut seq_menu = menu::Menu::new(EventType::MainMenu, EventType::NoActive,MenuIcon::Piano);
    seq_menu.add("PATCHES", MenuIcon::Patch, EventType::DrumPatchMenu, EventType::NoActive);
    seq_menu.add("SETTINGS", MenuIcon::Tune, EventType::NoMenu, EventType::NoActive);
    seq_menu.add("ABOUT", MenuIcon::Information, EventType::DrumAbout, EventType::NoActive);

    let mut drum_patch_menu = menu::Menu::new(EventType::DrumMenu, EventType::NoActive,MenuIcon::Patch);
    drum_patch_menu.add("TR 808", MenuIcon::Drum, EventType::NoMenu, EventType::PatchSelect);
    drum_patch_menu.add("TR 909", MenuIcon::Drum, EventType::NoMenu, EventType::PatchSelect);
    drum_patch_menu.add("WEIRD", MenuIcon::Drum, EventType::NoMenu, EventType::PatchSelect);

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
        match NAVIGATION_SIGNAL.receive().await {
                   EventType::DrumMenu => {
                    active_item = ActiveItem::Menu(&mut drum_menu);
                    active_item.activate();
                    },
                    EventType::NoMenu => {}
                    EventType::MainMenu => active_item = ActiveItem::Menu(&mut main_menu),
                    EventType::DrumPatchMenu => {
                        active_item = ActiveItem::Menu(&mut drum_patch_menu)
                    },
                     EventType::SynthPatchMenu => {
                        active_item = ActiveItem::Menu(&mut synth_patch_menu)
                    }
                    EventType::SynthMenu => {
                        active_item = ActiveItem::Menu(&mut synth_menu);
                        active_item.activate();
                    }
                    EventType::DrumAbout => active_item = ActiveItem::About(&mut about_drums_menu),
                    EventType::SeqMenu => active_item = ActiveItem::Menu(&mut seq_menu),
                    EventType::LttLMenuAbout => {
                        active_item = ActiveItem::About(&mut about_lttl_menu)
                    }
                    EventType::MenuUp => active_item.prev(),
                    EventType::MenuDown => active_item.next(),
                     EventType::MenuLeft=> active_item.back(),
                    EventType::MenuRight=> active_item.select(),
            _ => {}
        }
    }
}
