extern crate alloc;
use alloc::string::ToString;
use alloc::string::String;
use alloc::{vec, vec::Vec};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
// use embassy_futures::join::join;
use embassy_time::{Duration, Timer};
use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{DrawTarget, Point};
use embedded_graphics::primitives::{Line, PrimitiveStyle, StyledDrawable};
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::peripherals::Peripherals;
// use esp_println::println;
use esp_storage::FlashStorage;
use little_weirdo::synth::patch::Patch;
use little_weirdo_filesystem::WeirdoFileSystem;
use little_weirdo_gui::about::About;
use little_weirdo_gui::menu::{self, MenuIcon};
use little_weirdo_gui::settings::Settings;
use little_weirdo_gui::{
    ActiveItem, NAVIGATION_SIGNAL, Navigatable, NO_ACTIVITY_EVENT, NO_MENU_EVENT,
};
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};

use crate::I2c1Bus;
use crate::synth::SoundBank;
use crate::mapping::{VOLUME_CHANGED_EVENT_IDENTIFIER, WAVEFORM_CHANGED_EVENT_IDENTIFIER};

const WAVEFORM_SAMPLE_COUNT: usize = 600;
const WAVEFORM_BYTE_COUNT: usize = WAVEFORM_SAMPLE_COUNT * 2;
const WAVEFORM_NAMES_KEY: u32 = 600;

enum DisplaySetting {
    Waveform { name: String, samples: [i16; WAVEFORM_SAMPLE_COUNT] },
    Volume,
}

fn draw_waveform<D>(display: &mut D, samples: &[i16; WAVEFORM_SAMPLE_COUNT]) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let max_amplitude = samples
        .iter()
        .map(|sample| (*sample as i32).abs())
        .max()
        .unwrap_or(1)
        .max(1);
    let center_y = 47;
    let amplitude_height = 15;
    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    Line::new(Point::new(0, center_y), Point::new(127, center_y))
        .draw_styled(&line_style, display)?;

    for x in 0..127 {
        let sample_index = x as usize * (WAVEFORM_SAMPLE_COUNT - 1) / 127;
        let next_sample_index = (x as usize + 1) * (WAVEFORM_SAMPLE_COUNT - 1) / 127;
        let y = center_y
            - (samples[sample_index] as i32 * amplitude_height / max_amplitude) as i32;
        let next_y = center_y
            - (samples[next_sample_index] as i32 * amplitude_height / max_amplitude) as i32;
        Line::new(Point::new(x, y), Point::new(x + 1, next_y))
            .draw_styled(&line_style, display)?;
    }

    Ok(())
}

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

    let _ = filesystem.read_key_value(000, buf.as_mut_slice());
    let sound_bank: SoundBank = postcard::from_bytes(&buf).unwrap();
    let waveform_names = match filesystem.size_of_key_value(WAVEFORM_NAMES_KEY) {
        Ok(size) if size > 0 => {
            let mut names_buffer = vec![0u8; size as usize];
            match filesystem.read_key_value(WAVEFORM_NAMES_KEY, &mut names_buffer) {
                Ok(bytes_read) => match postcard::from_bytes::<Vec<String>>(
                    &names_buffer[..bytes_read as usize],
                ) {
                    Ok(names) => Some(names),
                    Err(error) => {
                        esp_println::println!("failed to decode waveform names: {:?}", error);
                        None
                    }
                },
                Err(error) => {
                    esp_println::println!("failed to read waveform names: {:?}", error);
                    None
                }
            }
        }
        Ok(_) => {
            esp_println::println!("waveform names are missing from the soundbank");
            None
        }
        Err(error) => {
            esp_println::println!("failed to locate waveform names: {:?}", error);
            None
        }
    };
 
    let mut main_menu = menu::Menu::new(NO_MENU_EVENT.clone(), NO_ACTIVITY_EVENT.clone(), MenuIcon::Sine);
/*     main_menu.add(
        "SYNTH".to_string(),
        MenuIcon::Sine,
        EventType::SynthMenu,
        EventType::NoActive,
    );
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
        EventType::SequencerModeActive,
    );

    main_menu.add(
        "MIDI".to_string(),
        MenuIcon::Midi,
        EventType::NoMenu,
        EventType::NoActive,
    );

    let mut control_settings = Settings::new(EventType::SynthMenu);
    control_settings.add("Vol".to_string());
    control_settings.add("Attack".to_string());
    control_settings.add("Decay".to_string());
    control_settings.add("Release".to_string());

    main_menu.add(
        "STATUS".to_string(),
        MenuIcon::Status,
        EventType::LttlStatus,
        EventType::NoActive,
    );
    */
    main_menu.add(
        "ABOUT".to_string(),
        MenuIcon::Information,
        NO_ACTIVITY_EVENT.clone(),
        NO_ACTIVITY_EVENT.clone(),
    );
/* 
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

    synth_menu.add(
        "CONTROLS".to_string(),
        MenuIcon::Tune,
        EventType::ControlSettings,
        EventType::NoActive,
    );

    synth_menu.add(
        "ABOUT".to_string(),
        MenuIcon::Information,
        EventType::NoMenu,
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
        "ABOUT".to_string(),
        MenuIcon::Information,
        EventType::NoMenu,
        EventType::NoActive,
    );

    let mut drum_patch_menu =
        menu::Menu::new(EventType::DrumMenu, EventType::NoActive, MenuIcon::Patch);
    for i in 0..sound_bank.amound_of_sampler_patches as u32 {
        let length_of_patch = filesystem
            .read_key_value(200 + i, &mut patch_read_buffer)
            .unwrap();
        let patch: little_weirdo::wavetable::patch::Patch =
            postcard::from_bytes(&patch_read_buffer[..length_of_patch as usize]).unwrap();
        drum_patch_menu.add(
            format!("{}", patch.name),
            MenuIcon::Drum,
            EventType::NoMenu,
            EventType::PatchSelect,
        );
    }

    let mut about_drums_menu = About::new(
        "LTTL Weirdo Drums is a collection of drum sounds for the LTTL Weirdo synthesizer."
            .to_string(),
        EventType::DrumMenu,
    );

    let mut about_lttl_menu = About::new(
        "LttL Hi SQυεΛκY Things Machine v1.0 ♫ §QυεΛΛΛΛκκ Cαη ħαρρεη α⊥ αηψ тıмε! ©'26 HST∞LAB ♥♪♥"
            .to_string(),
        EventType::MainMenu,
    );

    let mut main_status_menu = About::new(
        format!(
            "§TΛTU§ \n♫ SynthPatches {}x\n♫ SamplerPatches {}x",
            sound_bank.amount_of_patches, sound_bank.amound_of_sampler_patches
        )
        .to_string(),
        EventType::MainMenu,
    );

    let mut control_menu_active = false;

    */
    let mut active_item = ActiveItem::Menu(&mut main_menu);
    let mut volume_settings = Settings::new(NO_MENU_EVENT.clone());
    volume_settings.add("Volume".to_string());
    let mut display_setting = None;
    let mut waveform_bytes = [0u8; WAVEFORM_BYTE_COUNT];
    loop {
        let _ = display.clear(BinaryColor::Off);
        match &display_setting {
            Some(setting) => match setting {
                DisplaySetting::Waveform { name, samples } => {
                    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
                    let _ = Text::new("WAVEFORM", Point::new(0, 13), text_style).draw(&mut display);
                    let _ = Text::new(name, Point::new(0, 26), text_style).draw(&mut display);
                    let _ = draw_waveform(&mut display, samples);
                }
                DisplaySetting::Volume => {
                    let _ = volume_settings.draw(&mut display);
                }
            },
            None => {
                let _ = active_item.draw(&mut display);
            }
        }
        let _ = display.flush().await;
        let event = NAVIGATION_SIGNAL.receive().await;
        match event.event_identifier {
            WAVEFORM_CHANGED_EVENT_IDENTIFIER => {
                if let Some(name) = waveform_names
                    .as_ref()
                    .and_then(|names| names.get(event.id as usize))
                {
                    match filesystem.read_key_value(700 + event.id as u32, &mut waveform_bytes) {
                        Ok(bytes_read) if bytes_read >= WAVEFORM_BYTE_COUNT as u32 => {
                            let mut samples = [0i16; WAVEFORM_SAMPLE_COUNT];
                            for (index, sample) in samples.iter_mut().enumerate() {
                                let offset = index * 2;
                                *sample = i16::from_le_bytes([
                                    waveform_bytes[offset],
                                    waveform_bytes[offset + 1],
                                ]);
                            }
                            display_setting = Some(DisplaySetting::Waveform {
                                name: name.clone(),
                                samples,
                            });
                        }
                        Ok(bytes_read) => {
                            esp_println::println!(
                                "waveform {} has only {} sample bytes",
                                event.id,
                                bytes_read
                            );
                        }
                        Err(error) => {
                            esp_println::println!(
                                "failed to read waveform {}: {:?}",
                                event.id,
                                error
                            );
                        }
                    }
                } else {
                    esp_println::println!("waveform name missing for index {}", event.id);
                }
            }
            VOLUME_CHANGED_EVENT_IDENTIFIER => {
                volume_settings.update_bar(0, (event.value.min(127) * 100 / 127) as u8);
                display_setting = Some(DisplaySetting::Volume);
            }
            _ => {}
        }
      /* 
        match event.event {
            EventType::ControlSettings => {
                active_item = ActiveItem::Settings(&mut control_settings);
                control_menu_active = true;
            }
            EventType::DrumMenu => {
                active_item = ActiveItem::Menu(&mut drum_menu);
                active_item.activate();
            }
            EventType::NoMenu => {}
            EventType::MainMenu => {
                active_item = ActiveItem::Menu(&mut main_menu);
                control_menu_active = false;
            }
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
            EventType::PotMeterChange => {
                if control_menu_active {
                    active_item.update_bar(event.id, (event.value * 100 / 255) as u8);
                }
            }
            _ => {}
        }
        */
    }
}
