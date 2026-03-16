extern crate alloc;
use alloc::string::ToString;
use alloc::{format, vec};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_time::{Duration, Timer};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::peripherals::Peripherals;
use esp_println::println;
use esp_storage::FlashStorage;
use little_weirdo::synth::patch::Patch;
use little_weirdo_filesystem::WeirdoFileSystem;
use little_weirdo_gui::about::About;
use little_weirdo_gui::menu::{self, MenuIcon};
use little_weirdo_gui::{ActiveItem, DrawItem, EventType, NAVIGATION_SIGNAL, Navigatable};
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};

use crate::I2c1Bus;
use crate::synth::SoundBank;

#[embassy_executor::task]
pub async fn oled_task(i2c_bus: &'static I2c1Bus) {
    let i2c_dev = I2cDevice::new(i2c_bus);

    let interface = I2CDisplayInterface::new(i2c_dev);
    let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate180)
        .into_buffered_graphics_mode();

    let _ = display
        .init_with_addr_mode(ssd1306::command::AddrMode::Horizontal)
        .await;

    let _ = display.flush().await;

    Timer::after(Duration::from_millis(1000)).await;

    let storage;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        storage = FlashStorage::new(peripherals.FLASH);
    }

    let mut filesystem: WeirdoFileSystem<FlashStorage> =
        WeirdoFileSystem::new(storage, 0x00220000, 0x100000);

    //get settings

    let mut buf = vec![0u8; 256];

    filesystem.read_key_value(000, buf.as_mut_slice());
    let sound_bank: SoundBank = postcard::from_bytes(&buf).unwrap();

    let mut main_menu = menu::Menu::new(EventType::NoMenu, EventType::NoActive, MenuIcon::Sine);
    main_menu.add(
        "DRUMS".to_string(),
        MenuIcon::Music,
        EventType::DrumMenu,
        EventType::NoActive,
    );
    main_menu.add(
        "SEQUENCER".to_string(),
        MenuIcon::Piano,
        EventType::SeqMenu,
        EventType::NoActive,
    );
    main_menu.add(
        "SYNTH".to_string(),
        MenuIcon::Sine,
        EventType::SynthMenu,
        EventType::NoActive,
    );
    main_menu.add(
        "MIDI".to_string(),
        MenuIcon::Midi,
        EventType::NoMenu,
        EventType::NoActive,
    );
    main_menu.add(
        "SETTINGS".to_string(),
        MenuIcon::Tune,
        EventType::NoMenu,
        EventType::NoActive,
    );
    main_menu.add(
        "STATUS".to_string(),
        MenuIcon::Status,
        EventType::LttlStatus,
        EventType::NoActive,
    );
    main_menu.add(
        "ABOUT".to_string(),
        MenuIcon::Information,
        EventType::LttLMenuAbout,
        EventType::NoActive,
    );

    let mut synth_menu = menu::Menu::new(
        EventType::MainMenu,
        EventType::SynthModeActive,
        MenuIcon::Sine,
    );
    synth_menu.add(
        "PATCHES".to_string(),
        MenuIcon::Patch,
        EventType::SynthPatchMenu,
        EventType::NoActive,
    );

    let mut synth_patch_menu =
        menu::Menu::new(EventType::SynthMenu, EventType::NoActive, MenuIcon::Patch);

    let mut patch_read_buffer = vec![0u8; 300];
    for i in 0..sound_bank.amount_of_patches as u32 {
        let length_of_patch = filesystem
            .read_key_value(100 + i, &mut patch_read_buffer)
            .unwrap();
        let patch: Patch =
            postcard::from_bytes(&patch_read_buffer[..length_of_patch as usize]).unwrap();
        synth_patch_menu.add(
            format!("{} [{}]", patch.name, i),
            MenuIcon::Sine,
            EventType::NoMenu,
            EventType::PatchSelect,
        );
    }
 
    let mut drum_menu = menu::Menu::new(
        EventType::MainMenu,
        EventType::DrumModeActive,
        MenuIcon::Music,
    );
    drum_menu.add(
        "PATCHES".to_string(),
        MenuIcon::Patch,
        EventType::DrumPatchMenu,
        EventType::NoActive,
    );
    drum_menu.add(
        "SETTINGS".to_string(),
        MenuIcon::Tune,
        EventType::NoMenu,
        EventType::NoActive,
    );
    drum_menu.add(
        "ABOUT".to_string(),
        MenuIcon::Information,
        EventType::DrumAbout,
        EventType::NoActive,
    );

    let mut seq_menu = menu::Menu::new(EventType::MainMenu, EventType::NoActive, MenuIcon::Piano);
    seq_menu.add(
        "PATCHES".to_string(),
        MenuIcon::Patch,
        EventType::DrumPatchMenu,
        EventType::NoActive,
    );
    seq_menu.add(
        "SETTINGS".to_string(),
        MenuIcon::Tune,
        EventType::NoMenu,
        EventType::NoActive,
    );
    seq_menu.add(
        "ABOUT".to_string(),
        MenuIcon::Information,
        EventType::DrumAbout,
        EventType::NoActive,
    );

    let mut drum_patch_menu =
        menu::Menu::new(EventType::DrumMenu, EventType::NoActive, MenuIcon::Patch);
    for i in 0..sound_bank.amound_of_sampler_patches as u32 {
        let length_of_patch = filesystem
            .read_key_value(200 + i, &mut patch_read_buffer)
            .unwrap();
        let patch: little_weirdo::sampler::patch::Patch =
            postcard::from_bytes(&patch_read_buffer[..length_of_patch as usize]).unwrap();
        drum_patch_menu.add(
            format!("{}", patch.name),
            MenuIcon::Drum,
            EventType::NoMenu,
            EventType::PatchSelect,
        );
    }
  
    let mut about_drums_menu = About::new(
        "LTTL Weirdo Drums is a collection of drum sounds for the LTTL Weirdo synthesizer.".to_string(),
        EventType::DrumMenu,
    );

    let mut about_lttl_menu = About::new(
        "LttL Hi SQυεΛκY Things Machine v1.0 ♫ §QυεΛΛΛΛκκ Cαη ħαρρεη α⊥ αηψ тıмε! ©'26 HST∞LAB ♥♪♥".to_string(),
        EventType::MainMenu,
    );

    let mut main_status_menu = About::new(
        format!("§TΛTU§ \n♫ SynthPatches {}x\n♫ SamplerPatches {}x", sound_bank.amount_of_patches, sound_bank.amound_of_sampler_patches).to_string(),
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
            }
            EventType::NoMenu => {}
            EventType::MainMenu => active_item = ActiveItem::Menu(&mut main_menu),
            EventType::DrumPatchMenu => active_item = ActiveItem::Menu(&mut drum_patch_menu),
            EventType::SynthPatchMenu => active_item = ActiveItem::Menu(&mut synth_patch_menu),
            EventType::SynthMenu => {
                active_item = ActiveItem::Menu(&mut synth_menu);
                active_item.activate();
            }
            EventType::DrumAbout => active_item = ActiveItem::About(&mut about_drums_menu),
            EventType::SeqMenu => active_item = ActiveItem::Menu(&mut seq_menu),
            EventType::LttLMenuAbout => active_item = ActiveItem::About(&mut about_lttl_menu),
            EventType::LttlStatus => active_item = ActiveItem::About(&mut main_status_menu),
            EventType::MenuUp => active_item.prev(),
            EventType::MenuDown => active_item.next(),
            EventType::MenuLeft => active_item.back(),
            EventType::MenuRight => active_item.select(),
            _ => {}
        }
    }
}
