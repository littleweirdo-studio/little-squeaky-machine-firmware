extern crate alloc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::{format, vec, vec::Vec};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
// use embassy_futures::join::join;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::Drawable;
use embedded_graphics::mono_font::{MonoTextStyle, ascii::FONT_4X6, ascii::FONT_6X10};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::{DrawTarget, Point, Size};
use embedded_graphics::primitives::{
    Circle, Line, PrimitiveStyle, Rectangle, RoundedRectangle, StyledDrawable,
};
use embedded_graphics::text::Text;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::peripherals::Peripherals;
// use esp_println::println;
use esp_storage::FlashStorage;
use little_weirdo_filesystem::WeirdoFileSystem;
use little_weirdo_gui::about::About;
use little_weirdo_gui::{
    DrawItem, Event, NAVIGATION_SIGNAL, NO_ACTIVITY_EVENT, NO_MENU_EVENT, Navigatable,
    display::VerticalOffsetTarget,
    encoder_display::EncoderDisplay,
    menu::{Menu, MenuIcon},
    splash::Splash,
};
use ssd1306::prelude::DisplayRotation;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};

use crate::I2c1Bus;
use crate::VERSION;
use crate::led::INITIAL_BRIGHTNESS;
use crate::mapping::{
    BRIGHTNESS_CHANGED_EVENT_IDENTIFIER, CHORD_CHANGED_EVENT_IDENTIFIER, CHORD_NAMES,
    DELAY_CHANGED_EVENT_IDENTIFIER, DELAY_MODE_EVENT_ID, ENCODER_USED_EVENT_IDENTIFIER,
    ENVELOPE_CHANGED_EVENT_IDENTIFIER, ENVELOPE_MODE_EVENT_ID, envelope_time_ms,
    FM_CHANGED_EVENT_IDENTIFIER, FM_MODE_EVENT_ID, GUI_MENU_BUTTON_EVENT_IDENTIFIER,
    GUI_MENU_OPTION_SELECTED_EVENT_IDENTIFIER, GUI_MENU_ROTATION_EVENT_IDENTIFIER,
    INITIAL_ENVELOPE_VALUES, INITIAL_KEYPAD_OCTAVE_SHIFT, INITIAL_VOLUME,
    KEYPAD_OCTAVE_CHANGED_EVENT_IDENTIFIER, OVERDRIVE_CHANGED_EVENT_IDENTIFIER,
    OVERDRIVE_MODE_EVENT_ID, PATCH_SELECTION_EVENT_IDENTIFIER, VOLUME_CHANGED_EVENT_IDENTIFIER,
    WAVEFORM_CHANGED_EVENT_IDENTIFIER,
};
use crate::synth::SoundBank;

const WAVEFORM_SAMPLE_COUNT: usize = 600;
const WAVEFORM_BYTE_COUNT: usize = WAVEFORM_SAMPLE_COUNT * 2;
const WAVEFORM_NAMES_KEY: u32 = 600;
const WELCOME_SCREEN_DURATION: Duration = Duration::from_secs(3);
const ABOUT_MENU_OPTION_INDEX: usize = 4;
const ABOUT_TEXT: &str =
    "LttL Hi SQυεΛκY Things Machine v1.0 ♫ §QυεΛΛΛΛκκ Cαη ħαρρεη α⊥ αηψ тıмε! ©'26 HST∞LAB ♥♪♥";
const SCREEN_TRANSITION_OFFSETS: [i32; 8] = [-64, -48, -32, -16, -6, 1, -2, 0];
const SCREEN_TRANSITION_FRAME_DURATION: Duration = Duration::from_millis(18);
const GUI_MENU_OPTIONS: [(&str, MenuIcon); 6] = [
    ("Synth", MenuIcon::Sine),
    ("Sampler", MenuIcon::Patch),
    ("Drums", MenuIcon::Drum),
    ("Settings", MenuIcon::Tune),
    ("About", MenuIcon::Information),
    ("Status", MenuIcon::Status),
];
const LOGO: &[u8; 512] = include_bytes!("data/logo.img");

fn create_gui_menu() -> Menu {
    let mut menu = Menu::new(
        NO_MENU_EVENT.clone(),
        NO_ACTIVITY_EVENT.clone(),
        MenuIcon::Sine,
    );
    for (title, icon) in GUI_MENU_OPTIONS {
        menu.add(
            title.to_string(),
            icon,
            NO_MENU_EVENT.clone(),
            Event {
                event_identifier: GUI_MENU_OPTION_SELECTED_EVENT_IDENTIFIER,
                id: 0,
                value: 0,
            },
        );
    }
    menu
}

fn create_patch_menu(filesystem: &mut WeirdoFileSystem<FlashStorage>, patch_count: usize) -> Menu {
    let mut menu = Menu::new(
        NO_MENU_EVENT.clone(),
        NO_ACTIVITY_EVENT.clone(),
        MenuIcon::Patch,
    );
    for index in 0..patch_count {
        let patch_index = index as u8;
        let name = read_patch_name(filesystem, patch_index)
            .unwrap_or_else(|| format!("Patch {}", index + 1));
        menu.add(
            format!("{}", name),
            MenuIcon::Patch,
            NO_MENU_EVENT.clone(),
            Event {
                event_identifier: GUI_MENU_OPTION_SELECTED_EVENT_IDENTIFIER,
                id: 0,
                value: 0,
            },
        );
    }
    menu
}

fn draw_screen<D>(
    display: &mut D,
    welcome_visible: bool,
    gui_menu: Option<&Menu>,
    display_setting: &Option<DisplaySetting>,
    volume: u8,
    keypad_octave_shift: i8,
    brightness: u8,
    envelope_values: &[u8; 4],
    overdrive_enabled: Option<bool>,
    overdrive_kind: Option<u8>,
    overdrive_threshold: Option<u16>,
    delay_enabled: Option<bool>,
    delay_time_ms: Option<u16>,
    delay_mix_percentage: Option<u8>,
    delay_feedback_percentage: Option<u8>,
    fm_carrier_waveform: Option<u8>,
    fm_modulator_waveform: Option<u8>,
    fm_ratio: Option<u8>,
    fm_depth_hz: Option<u16>,
) where
    D: DrawTarget<Color = BinaryColor>,
{
    if let Some(menu) = gui_menu {
        let _ = menu.draw(display);
        return;
    }

    if welcome_visible {
        let _ = draw_welcome(display);
        return;
    }

    match display_setting {
        Some(DisplaySetting::Waveform { name, samples }) => {
            // let _ = EncoderDisplay::new(1, "Waveform", None, Point::new(7, 15)).draw(display);
            let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
            let _ = Text::new(name, Point::new(7, 15), text_style).draw(display);
            let _ = draw_waveform(display, samples);
            let _ = draw_screen_border(display);
        }
        Some(DisplaySetting::Controls) => {
            let _ = draw_controls(display, volume, keypad_octave_shift, brightness);
        }
        Some(DisplaySetting::About(about)) => {
            let _ = about.draw(display);
        }
        Some(DisplaySetting::Chord(name)) => {
            let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
            let _ = draw_chord_icon(display);
            let _ = Text::new("CHORD", Point::new(48, 22), text_style).draw(display);
            let _ = Text::new(name, Point::new(48, 40), text_style).draw(display);
            let _ = draw_screen_border(display);
        }
        Some(DisplaySetting::Patch {
            name,
            index,
            browsing,
        }) => {
            let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
            let _ = Text::new("PATCH SELECT", Point::new(7, 16), text_style).draw(display);
            let _ = Text::new(name, Point::new(7, 31), text_style).draw(display);
            let _ = Text::new(
                &format!("Patch {}", *index as u16 + 1),
                Point::new(7, 45),
                text_style,
            )
            .draw(display);
            let footer = if *browsing {
                "Press 0 to load"
            } else {
                "Patch loaded"
            };
            let _ = Text::new(footer, Point::new(7, 56), text_style).draw(display);
            let _ = draw_screen_border(display);
        }
        Some(DisplaySetting::Envelope) => {
            let _ = draw_envelope(display, envelope_values);
        }
        Some(DisplaySetting::Overdrive) => {
            let _ = draw_overdrive(
                display,
                overdrive_enabled,
                overdrive_kind,
                overdrive_threshold,
            );
        }
        Some(DisplaySetting::Delay) => {
            let _ = draw_delay(
                display,
                delay_enabled,
                delay_time_ms,
                delay_mix_percentage,
                delay_feedback_percentage,
            );
        }
        Some(DisplaySetting::Fm) => {
            let carrier_waveform_text = fm_carrier_waveform
                .map(|value| format!("{}", value))
                .unwrap_or_else(|| "--".to_string());
            let modulator_waveform_text = fm_modulator_waveform
                .map(|value| format!("{}", value))
                .unwrap_or_else(|| "--".to_string());
            let ratio_text = fm_ratio
                .map(|value| format!("{}x", value.max(1)))
                .unwrap_or_else(|| "--".to_string());
            let depth_text = fm_depth_hz
                .map(|value| format!("{}", value))
                .unwrap_or_else(|| "--".to_string());
            let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
            let _ = Text::new("FM SYNTH", Point::new(7, 16), text_style).draw(display);
            let _ = EncoderDisplay::new(
                0,
                "Carrier",
                Some(&carrier_waveform_text),
                Point::new(7, 27),
            )
            .draw(display);
            let _ = EncoderDisplay::new(
                1,
                "Mod Wave",
                Some(&modulator_waveform_text),
                Point::new(7, 37),
            )
            .draw(display);
            let _ =
                EncoderDisplay::new(2, "Ratio", Some(&ratio_text), Point::new(7, 47)).draw(display);
            let _ =
                EncoderDisplay::new(3, "Depth", Some(&depth_text), Point::new(7, 57)).draw(display);
            let _ = draw_screen_border(display);
        }
        None => {
            let _ = draw_controls(display, volume, keypad_octave_shift, brightness);
        }
    }
}

fn read_patch_name(filesystem: &mut WeirdoFileSystem<FlashStorage>, index: u8) -> Option<String> {
    let key = 100 + index as u32;
    let size = match filesystem.size_of_key_value(key) {
        Ok(size) if size > 0 => size as usize,
        Ok(_) => {
            esp_println::println!("patch {} is empty in the soundbank", index);
            return None;
        }
        Err(error) => {
            esp_println::println!("failed to locate patch {}: {:?}", index, error);
            return None;
        }
    };
    let mut buffer = vec![0u8; size];
    let bytes_read = match filesystem.read_key_value(key, &mut buffer) {
        Ok(bytes_read) => bytes_read as usize,
        Err(error) => {
            esp_println::println!("failed to read patch {}: {:?}", index, error);
            return None;
        }
    };
    match postcard::from_bytes::<little_weirdo::synth::patch::Patch>(&buffer[..bytes_read]) {
        Ok(patch) => Some(patch.name),
        Err(error) => {
            esp_println::println!("failed to decode patch {}: {:?}", index, error);
            None
        }
    }
}

fn draw_screen_border<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(0, 0), Size::new(128, 64)),
        Size::new(5, 5),
    )
    .draw_styled(&line_style, display)
}

enum DisplaySetting {
    Waveform {
        name: String,
        samples: [i16; WAVEFORM_SAMPLE_COUNT],
    },
    Controls,
    About(About),
    Chord(&'static str),
    Patch {
        name: String,
        index: u8,
        browsing: bool,
    },
    Envelope,
    Overdrive,
    Delay,
    Fm,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DisplayScreenKind {
    Waveform,
    Controls,
    About,
    Chord,
    PatchBrowser,
    Envelope,
    Overdrive,
    Delay,
    Fm,
}

fn display_screen_kind(setting: Option<&DisplaySetting>) -> DisplayScreenKind {
    match setting {
        Some(DisplaySetting::Waveform { .. }) => DisplayScreenKind::Waveform,
        Some(DisplaySetting::Controls) | None => DisplayScreenKind::Controls,
        Some(DisplaySetting::About(_)) => DisplayScreenKind::About,
        Some(DisplaySetting::Chord(_)) => DisplayScreenKind::Chord,
        Some(DisplaySetting::Patch { .. }) => DisplayScreenKind::PatchBrowser,
        Some(DisplaySetting::Envelope) => DisplayScreenKind::Envelope,
        Some(DisplaySetting::Overdrive) => DisplayScreenKind::Overdrive,
        Some(DisplaySetting::Delay) => DisplayScreenKind::Delay,
        Some(DisplaySetting::Fm) => DisplayScreenKind::Fm,
    }
}

fn is_patch_screen(setting: Option<&DisplaySetting>) -> bool {
    matches!(setting, Some(DisplaySetting::Patch { .. }))
}

fn draw_controls<D>(
    display: &mut D,
    volume: u8,
    octave_shift: i8,
    brightness: u8,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let volume_percent = volume.min(127) as u16 * 100 / 127;
    let brightness_percent = brightness as u16 * 100 / u8::MAX as u16;

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("CONTROLS", Point::new(7, 12), text_style).draw(display)?;
    EncoderDisplay::new(
        0,
        "Volume",
        Some(&format!("{}%", volume_percent)),
        Point::new(7, 23),
    )
    .draw(display)?;
    EncoderDisplay::new(1, "Waveform", None, Point::new(7, 33)).draw(display)?;
    EncoderDisplay::new(
        2,
        "Octave",
        Some(&format!("{:+}", octave_shift)),
        Point::new(7, 43),
    )
    .draw(display)?;
    EncoderDisplay::new(
        3,
        "Brightness",
        Some(&format!("{}%", brightness_percent)),
        Point::new(7, 53),
    )
    .draw(display)?;
   draw_screen_border(display)
}

fn draw_welcome<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let mut logo_splash = Splash::new(embedded_graphics::image::ImageRaw::new(LOGO, 64));

    logo_splash.draw(display)?;

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("lsm", Point::new(8, 45), text_style).draw(display)?;
    Text::new(&format!("v_{}", VERSION), Point::new(8, 55), text_style).draw(display)?;

    draw_screen_border(display)
}

fn draw_envelope<D>(display: &mut D, values: &[u8; 4]) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let attack_ms = envelope_time_ms(values[0]);
    let decay_ms = envelope_time_ms(values[1]);
    let release_ms = envelope_time_ms(values[3]);
    let format_time = |time_ms: u16| {
        if time_ms < 1000 {
            format!("{time_ms}ms")
        } else {
            format!("{}.{:01}s", time_ms / 1000, time_ms % 1000 / 100)
        }
    };
    let plot_left = 8;
    let plot_right = 120;
    let plot_top = 14;
    let baseline = 22;
    let sustain_y = baseline - (values[2] as i32 * (baseline - plot_top) / 100);

    let attack = attack_ms as u32;
    let decay = decay_ms as u32;
    let release = release_ms as u32;
    let total_time = attack + decay + release;
    let adjustable_width = (plot_right - plot_left - 28 - 24) as u32;
    let attack_extra = adjustable_width * attack / total_time;
    let decay_extra = adjustable_width * decay / total_time;
    let release_extra = adjustable_width - attack_extra - decay_extra;
    let attack_width = 8 + attack_extra as i32;
    let decay_width = 8 + decay_extra as i32;
    let sustain_width = 28;
    let release_width = 8 + release_extra as i32;

    let attack_end = plot_left + attack_width;
    let decay_end = attack_end + decay_width;
    let sustain_end = decay_end + sustain_width;
    let release_end = sustain_end + release_width;
    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    Line::new(
        Point::new(plot_left, baseline),
        Point::new(attack_end, plot_top),
    )
    .draw_styled(&line_style, display)?;
    Line::new(
        Point::new(attack_end, plot_top),
        Point::new(decay_end, sustain_y),
    )
    .draw_styled(&line_style, display)?;
    Line::new(
        Point::new(decay_end, sustain_y),
        Point::new(sustain_end, sustain_y),
    )
    .draw_styled(&line_style, display)?;
    Line::new(
        Point::new(sustain_end, sustain_y),
        Point::new(release_end, baseline),
    )
    .draw_styled(&line_style, display)?;

    Text::new("ADSR", Point::new(7, 12), text_style).draw(display)?;
    EncoderDisplay::new(
        0,
        "Attack",
        Some(&format_time(attack_ms)),
        Point::new(7, 31),
    )
    .draw(display)?;
    EncoderDisplay::new(1, "Decay", Some(&format_time(decay_ms)), Point::new(7, 40))
        .draw(display)?;
    EncoderDisplay::new(
        2,
        "Sustain",
        Some(&format!("{}%", values[2])),
        Point::new(7, 49),
    )
    .draw(display)?;
    EncoderDisplay::new(
        3,
        "Release",
        Some(&format_time(release_ms)),
        Point::new(7, 58),
    )
    .draw(display)?;
    draw_screen_border(display)
}

fn draw_overdrive<D>(
    display: &mut D,
    enabled: Option<bool>,
    kind: Option<u8>,
    threshold: Option<u16>,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let enabled_text = match enabled {
        Some(true) => "ON",
        Some(false) => "OFF",
        None => "--",
    };
    let kind_text = match kind {
        Some(0) => "Hard",
        Some(1) => "Soft",
        Some(2) => "Softer",
        _ => "--",
    };
    let threshold_text = threshold
        .map(|value| format!("{}", value))
        .unwrap_or_else(|| "--".to_string());

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("OVERDRIVE", Point::new(7, 16), text_style).draw(display)?;
    EncoderDisplay::new(0, "Enable", Some(enabled_text), Point::new(7, 29)).draw(display)?;
    EncoderDisplay::new(1, "Kind", Some(kind_text), Point::new(7, 41)).draw(display)?;
    EncoderDisplay::new(2, "Threshold", Some(&threshold_text), Point::new(7, 53)).draw(display)?;
    draw_screen_border(display)
}

fn draw_delay<D>(
    display: &mut D,
    enabled: Option<bool>,
    time_ms: Option<u16>,
    mix_percentage: Option<u8>,
    feedback_percentage: Option<u8>,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let enabled_text = match enabled {
        Some(true) => "ON",
        Some(false) => "OFF",
        None => "--",
    };
    let time_text = time_ms
        .map(|value| format!("{}ms", value))
        .unwrap_or_else(|| "--".to_string());
    let mix_text = mix_percentage
        .map(|value| format!("{}%", value))
        .unwrap_or_else(|| "--".to_string());
    let feedback_text = feedback_percentage
        .map(|value| {
            if value == 0 {
                "OFF".to_string()
            } else {
                format!("{}%", value)
            }
        })
        .unwrap_or_else(|| "--".to_string());

    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    Text::new("DELAY", Point::new(7, 16), text_style).draw(display)?;
    EncoderDisplay::new(0, "Enable", Some(enabled_text), Point::new(7, 27)).draw(display)?;
    EncoderDisplay::new(1, "Time", Some(&time_text), Point::new(7, 37)).draw(display)?;
    EncoderDisplay::new(2, "Mix", Some(&mix_text), Point::new(7, 47)).draw(display)?;
    EncoderDisplay::new(3, "Feedback", Some(&feedback_text), Point::new(7, 57)).draw(display)?;
    draw_screen_border(display)
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
    let plot_left = 7;
    let plot_width = 114;
    let center_y = 39;
    let amplitude_height = 19;
    let line_style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    Line::new(
        Point::new(plot_left, center_y),
        Point::new(plot_left + plot_width, center_y),
    )
    .draw_styled(&line_style, display)?;

    for x in 0..plot_width {
        let sample_index = x as usize * (WAVEFORM_SAMPLE_COUNT - 1) / plot_width as usize;
        let next_sample_index =
            (x as usize + 1) * (WAVEFORM_SAMPLE_COUNT - 1) / plot_width as usize;
        let y = center_y - (samples[sample_index] as i32 * amplitude_height / max_amplitude);
        let next_y =
            center_y - (samples[next_sample_index] as i32 * amplitude_height / max_amplitude);
        let plot_x = plot_left + x;
        Line::new(Point::new(plot_x, y), Point::new(plot_x + 1, next_y))
            .draw_styled(&line_style, display)?;
    }

    Ok(())
}

fn draw_chord_icon<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let note_style = PrimitiveStyle::with_fill(BinaryColor::On);
    let stem_style = PrimitiveStyle::with_stroke(BinaryColor::On, 2);

    Circle::new(Point::new(8, 38), 8).draw_styled(&note_style, display)?;
    Circle::new(Point::new(19, 30), 8).draw_styled(&note_style, display)?;
    Circle::new(Point::new(30, 22), 8).draw_styled(&note_style, display)?;

    Line::new(Point::new(15, 41), Point::new(15, 17)).draw_styled(&stem_style, display)?;
    Line::new(Point::new(26, 33), Point::new(26, 12)).draw_styled(&stem_style, display)?;
    Line::new(Point::new(37, 25), Point::new(37, 7)).draw_styled(&stem_style, display)?;
    Line::new(Point::new(15, 17), Point::new(37, 7)).draw_styled(&stem_style, display)?;

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
                Ok(bytes_read) => {
                    match postcard::from_bytes::<Vec<String>>(&names_buffer[..bytes_read as usize])
                    {
                        Ok(names) => Some(names),
                        Err(error) => {
                            esp_println::println!("failed to decode waveform names: {:?}", error);
                            None
                        }
                    }
                }
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
    let patch_menu_count = (sound_bank.amount_of_patches as usize).min(u8::MAX as usize + 1);
    let mut patch_menu = create_patch_menu(&mut filesystem, patch_menu_count);

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
    let mut welcome_visible = true;
    let welcome_deadline = Instant::now() + WELCOME_SCREEN_DURATION;
    let mut gui_menu = create_gui_menu();
    let mut gui_menu_open = false;
    let mut gui_menu_selected_index = 0;
    let mut patch_menu_active = false;
    let mut patch_menu_selected_index = 0;
    let mut display_setting = Some(DisplaySetting::Controls);
    let mut volume = INITIAL_VOLUME;
    let mut keypad_octave_shift = INITIAL_KEYPAD_OCTAVE_SHIFT;
    let mut brightness = INITIAL_BRIGHTNESS;
    let mut envelope_values = INITIAL_ENVELOPE_VALUES;
    let mut envelope_edit_mode = false;
    let mut overdrive_edit_mode = false;
    let mut overdrive_enabled = None;
    let mut overdrive_kind = None;
    let mut overdrive_threshold = None;
    let mut delay_edit_mode = false;
    let mut fm_edit_mode = false;
    let mut delay_enabled = None;
    let mut delay_time_ms = None;
    let mut delay_mix_percentage = None;
    let mut delay_feedback_percentage = None;
    let mut fm_carrier_waveform = None;
    let mut fm_modulator_waveform = None;
    let mut fm_ratio = None;
    let mut fm_depth_hz = None;
    let mut waveform_bytes = [0u8; WAVEFORM_BYTE_COUNT];
    let mut screen_transition = false;
    loop {
        if core::mem::take(&mut screen_transition) {
            for offset in SCREEN_TRANSITION_OFFSETS {
                let _ = display.clear(BinaryColor::Off);
                let mut offset_display = VerticalOffsetTarget::new(&mut display, offset);
                draw_screen(
                    &mut offset_display,
                    welcome_visible,
                    if patch_menu_active {
                        Some(&patch_menu)
                    } else {
                        gui_menu_open.then_some(&gui_menu)
                    },
                    &display_setting,
                    volume,
                    keypad_octave_shift,
                    brightness,
                    &envelope_values,
                    overdrive_enabled,
                    overdrive_kind,
                    overdrive_threshold,
                    delay_enabled,
                    delay_time_ms,
                    delay_mix_percentage,
                    delay_feedback_percentage,
                    fm_carrier_waveform,
                    fm_modulator_waveform,
                    fm_ratio,
                    fm_depth_hz,
                );
                let _ = display.flush().await;
                Timer::after(SCREEN_TRANSITION_FRAME_DURATION).await;
            }
        } else {
            let _ = display.clear(BinaryColor::Off);
            draw_screen(
                &mut display,
                welcome_visible,
                if patch_menu_active {
                    Some(&patch_menu)
                } else {
                    gui_menu_open.then_some(&gui_menu)
                },
                &display_setting,
                volume,
                keypad_octave_shift,
                brightness,
                &envelope_values,
                overdrive_enabled,
                overdrive_kind,
                overdrive_threshold,
                delay_enabled,
                delay_time_ms,
                delay_mix_percentage,
                delay_feedback_percentage,
                fm_carrier_waveform,
                fm_modulator_waveform,
                fm_ratio,
                fm_depth_hz,
            );
            let _ = display.flush().await;
        }
        let mut events: [Option<Event>; 50] = core::array::from_fn(|_| None);
        if welcome_visible {
            match select(NAVIGATION_SIGNAL.receive(), Timer::at(welcome_deadline)).await {
                Either::First(event) => events[0] = Some(event),
                Either::Second(()) => {
                    welcome_visible = false;
                    screen_transition = true;
                    continue;
                }
            }
        } else {
            events[0] = Some(NAVIGATION_SIGNAL.receive().await);
        }
        let mut event_count = 1;
        while event_count < events.len() {
            match NAVIGATION_SIGNAL.try_receive() {
                Ok(event) => {
                    events[event_count] = Some(event);
                    event_count += 1;
                }
                Err(_) => break,
            }
        }

        let was_welcome_visible = welcome_visible;
        let previous_screen = display_screen_kind(display_setting.as_ref());
        let previous_gui_menu_open = gui_menu_open;
        let previous_patch_menu_active = patch_menu_active;
        let mut about_selected = false;
        for event in events[..event_count].iter().flatten() {
            if event.event_identifier == ENCODER_USED_EVENT_IDENTIFIER {
                welcome_visible = false;
                continue;
            }
            match event.event_identifier {
                GUI_MENU_BUTTON_EVENT_IDENTIFIER => {
                    if patch_menu_active {
                        patch_menu.select();
                    } else {
                        gui_menu_open = event.value != 0;
                        if gui_menu_open {
                            welcome_visible = false;
                        } else {
                            gui_menu.select();
                            about_selected = gui_menu_selected_index == ABOUT_MENU_OPTION_INDEX;
                        }
                    }
                }
                GUI_MENU_ROTATION_EVENT_IDENTIFIER => {
                    if patch_menu_active {
                        if patch_menu_count > 0 {
                            patch_menu_selected_index = if event.value == 1 {
                                (patch_menu_selected_index + 1) % patch_menu_count
                            } else if patch_menu_selected_index == 0 {
                                patch_menu_count - 1
                            } else {
                                patch_menu_selected_index - 1
                            };
                            let _ = patch_menu.set_selected_index(patch_menu_selected_index);
                        }
                    } else if gui_menu_open {
                        if event.value == 1 {
                            gui_menu.next();
                            gui_menu_selected_index =
                                (gui_menu_selected_index + 1) % GUI_MENU_OPTIONS.len();
                        } else {
                            gui_menu.prev();
                            gui_menu_selected_index = if gui_menu_selected_index == 0 {
                                GUI_MENU_OPTIONS.len() - 1
                            } else {
                                gui_menu_selected_index - 1
                            };
                        }
                    }
                }
                GUI_MENU_OPTION_SELECTED_EVENT_IDENTIFIER => {
                    if patch_menu_active {
                        esp_println::println!("unexpected patch menu selection event {}", event.id);
                    } else {
                        if event.id as usize >= GUI_MENU_OPTIONS.len()
                            || !gui_menu.set_selected_index(event.id as usize)
                        {
                            esp_println::println!(
                                "selected menu index {} is out of range",
                                event.id
                            );
                        } else {
                            gui_menu_selected_index = event.id as usize;
                            about_selected |= gui_menu_selected_index == ABOUT_MENU_OPTION_INDEX;
                            if gui_menu_selected_index == 0 {
                                if event.value == u16::MAX || patch_menu_count == 0 {
                                    esp_println::println!(
                                        "synth patch selection unavailable: no patches found"
                                    );
                                } else {
                                    patch_menu_selected_index =
                                        (event.value as usize).min(patch_menu_count - 1);
                                    let _ =
                                        patch_menu.set_selected_index(patch_menu_selected_index);
                                    patch_menu_active = true;
                                }
                            }
                        }
                    }
                }
                WAVEFORM_CHANGED_EVENT_IDENTIFIER => {
                    if let Some(name) = waveform_names
                        .as_ref()
                        .and_then(|names| names.get(event.id as usize))
                    {
                        match filesystem.read_key_value(700 + event.id as u32, &mut waveform_bytes)
                        {
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
                PATCH_SELECTION_EVENT_IDENTIFIER => {
                    if event.id as u16 >= sound_bank.amount_of_patches {
                        esp_println::println!(
                            "patch selection index {} is outside the soundbank",
                            event.id
                        );
                        continue;
                    }
                    if event.value != 1 && event.value != 2 {
                        esp_println::println!("invalid patch selection state {}", event.value);
                        continue;
                    }

                    welcome_visible = false;
                    if event.value == 2 {
                        patch_menu_active = false;
                    }
                    let name = read_patch_name(&mut filesystem, event.id)
                        .unwrap_or_else(|| format!("Patch {}", event.id as u16 + 1));
                    display_setting = Some(DisplaySetting::Patch {
                        name,
                        index: event.id,
                        browsing: event.value == 1,
                    });
                }
                VOLUME_CHANGED_EVENT_IDENTIFIER => {
                    volume = event.value.min(127) as u8;
                    if !is_patch_screen(display_setting.as_ref()) {
                        display_setting = Some(DisplaySetting::Controls);
                    }
                }
                BRIGHTNESS_CHANGED_EVENT_IDENTIFIER => {
                    brightness = event.value.min(u8::MAX as u16) as u8;
                    if !is_patch_screen(display_setting.as_ref()) {
                        display_setting = Some(DisplaySetting::Controls);
                    }
                }
                CHORD_CHANGED_EVENT_IDENTIFIER => {
                    let name = CHORD_NAMES
                        .get(event.id as usize)
                        .copied()
                        .unwrap_or("No chord");
                    display_setting = Some(DisplaySetting::Chord(name));
                }
                KEYPAD_OCTAVE_CHANGED_EVENT_IDENTIFIER => {
                    keypad_octave_shift = event.value as u8 as i8;
                    if !is_patch_screen(display_setting.as_ref()) {
                        display_setting = Some(DisplaySetting::Controls);
                    }
                }
                ENVELOPE_CHANGED_EVENT_IDENTIFIER => {
                    if event.id == ENVELOPE_MODE_EVENT_ID {
                        envelope_edit_mode = event.value != 0;
                        display_setting = if envelope_edit_mode {
                            Some(DisplaySetting::Envelope)
                        } else {
                            None
                        };
                    } else if let Some(value) = envelope_values.get_mut(event.id as usize) {
                        *value = event.value.min(if event.id == 2 { 100 } else { 127 }) as u8;
                        if envelope_edit_mode {
                            display_setting = Some(DisplaySetting::Envelope);
                        }
                    }
                }
                OVERDRIVE_CHANGED_EVENT_IDENTIFIER => match event.id {
                    OVERDRIVE_MODE_EVENT_ID => {
                        overdrive_edit_mode = event.value != 0;
                        display_setting = if overdrive_edit_mode {
                            Some(DisplaySetting::Overdrive)
                        } else {
                            None
                        };
                    }
                    0 => {
                        overdrive_enabled = Some(event.value != 0);
                        if overdrive_edit_mode {
                            display_setting = Some(DisplaySetting::Overdrive);
                        }
                    }
                    1 => {
                        overdrive_kind = Some(event.value.min(2) as u8);
                        if overdrive_edit_mode {
                            display_setting = Some(DisplaySetting::Overdrive);
                        }
                    }
                    2 => {
                        overdrive_threshold = Some(event.value);
                        if overdrive_edit_mode {
                            display_setting = Some(DisplaySetting::Overdrive);
                        }
                    }
                    _ => {}
                },
                DELAY_CHANGED_EVENT_IDENTIFIER => match event.id {
                    DELAY_MODE_EVENT_ID => {
                        delay_edit_mode = event.value != 0;
                        display_setting = if delay_edit_mode {
                            Some(DisplaySetting::Delay)
                        } else {
                            None
                        };
                    }
                    0 => {
                        delay_enabled = Some(event.value != 0);
                        if delay_edit_mode {
                            display_setting = Some(DisplaySetting::Delay);
                        }
                    }
                    1 => {
                        delay_time_ms = Some(event.value);
                        if delay_edit_mode {
                            display_setting = Some(DisplaySetting::Delay);
                        }
                    }
                    2 => {
                        delay_mix_percentage = Some(event.value.min(100) as u8);
                        if delay_edit_mode {
                            display_setting = Some(DisplaySetting::Delay);
                        }
                    }
                    3 => {
                        delay_feedback_percentage = Some(event.value.min(100) as u8);
                        if delay_edit_mode {
                            display_setting = Some(DisplaySetting::Delay);
                        }
                    }
                    _ => {}
                },
                FM_CHANGED_EVENT_IDENTIFIER => match event.id {
                    FM_MODE_EVENT_ID => {
                        fm_edit_mode = event.value != 0;
                        display_setting = if fm_edit_mode {
                            Some(DisplaySetting::Fm)
                        } else {
                            None
                        };
                    }
                    0 => {
                        fm_carrier_waveform = Some(event.value.min(u8::MAX as u16) as u8);
                        if fm_edit_mode {
                            display_setting = Some(DisplaySetting::Fm);
                        }
                    }
                    1 => {
                        fm_modulator_waveform = Some(event.value.min(u8::MAX as u16) as u8);
                        if fm_edit_mode {
                            display_setting = Some(DisplaySetting::Fm);
                        }
                    }
                    2 => {
                        fm_ratio = Some(event.value.min(u8::MAX as u16) as u8);
                        if fm_edit_mode {
                            display_setting = Some(DisplaySetting::Fm);
                        }
                    }
                    3 => {
                        fm_depth_hz = Some(event.value);
                        if fm_edit_mode {
                            display_setting = Some(DisplaySetting::Fm);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        if about_selected {
            display_setting = Some(DisplaySetting::About(About::new(
                ABOUT_TEXT.to_string(),
                NO_MENU_EVENT.clone(),
            )));
        }
        screen_transition = (was_welcome_visible != welcome_visible)
            || previous_screen != display_screen_kind(display_setting.as_ref())
            || previous_gui_menu_open != gui_menu_open
            || previous_patch_menu_active != patch_menu_active;
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
