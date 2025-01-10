#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use alloc::format;
use alloc::rc::Rc;
use alloc::vec::Vec;
use embassy_executor::Spawner;
extern crate alloc;
use embassy_usb::Builder;
use embedded_graphics::mono_font::iso_8859_3::FONT_10X20;
use esp_alloc as _;
use embassy_time::Duration;
use embassy_time::Timer;
use embedded_graphics::mono_font::mapping::StrGlyphMapping;
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

use esp_backtrace as _;
use esp_alloc as _;
use esp_hal::analog::adc;
use esp_hal::analog::adc::Adc;
use esp_hal::analog::adc::AdcCalBasic;
use esp_hal::analog::adc::AdcCalLine;
use esp_hal::analog::adc::AdcConfig;
use esp_hal::analog::adc::Attenuation;
use esp_hal::dma::Dma;
use esp_hal::dma::DmaPriority;
use esp_hal::dma_circular_buffers;
use esp_hal::gpio::AnalogPin;
use esp_hal::gpio::GpioPin;
use esp_hal::gpio::Input;
use esp_hal::i2c::master::Operation;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::i2s::master::DataFormat;
use esp_hal::i2s::master::I2s;
use esp_hal::i2s::master::Standard;
use esp_hal::otg_fs::asynch::Driver;
use esp_hal::otg_fs::Usb;
use esp_hal::psram::{self, PsramSize};
use esp_hal::peripheral;
use esp_hal::peripheral::Peripheral;
use esp_hal::peripherals::ADC1;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::{
    prelude::*,
    timer::{timg::TimerGroup, AnyTimer},
};
use embedded_graphics::{image::Image, prelude::*};
use esp_println::println;
use esp_storage::FlashStorage;

use embedded_storage::{ReadStorage, Storage};

use little_weirdo::synth::sampler::BoxedSample;
use little_weirdo::synth::{
    self,
    data::wavetables::{BoxedWavetable, BoxedWavetables},
    effects::{
        filter::{FilterConfig, KindOfFilter},
        overdrive::{KindOfOverdrive, OverdriveConfiguration},
    },
    envelope::EnvelopConfiguration,
    mixer::MixerConfiguration,
    patch::{Patch, SynthConfiguration, SynthMode},
    router::{RoutingConfiguration, VoiceToEnvelopRoute, VoiceToLFORoute},
    wavetable_oscillator::{WaveTableLoFreqOscillatorConfig, WaveTableOscillatorConfig},
};


pub const SGTL5000_I2C_ADDR:u8 = 0x0A;

//
pub const SGTL5000_CHIP_ID: u16 = 0x0000;
pub const SGTL5000_CHIP_DIG_POWER: u16 = 0x0002;
pub const SGTL5000_CHIP_CLK_CTRL: u16 = 0x0004;
pub const SGTL5000_CHIP_I2S_CTRL: u16 = 0x0006;
pub const SGTL5000_CHIP_SSS_CTRL: u16 = 0x000A;
pub const SGTL5000_CHIP_ADCDAC_CTRL: u16 = 0x000E;
pub const SGTL5000_CHIP_DAC_VOL: u16 = 0x0010;
pub const SGTL5000_CHIP_PAD_STRENGTH: u16 = 0x0014;
pub const SGTL5000_CHIP_ANA_ADC_CTRL: u16 = 0x0020;
pub const SGTL5000_CHIP_ANA_HP_CTRL: u16 = 0x0022;
pub const SGTL5000_CHIP_ANA_CTRL: u16 = 0x0024;
pub const SGTL5000_CHIP_LINREG_CTRL: u16 = 0x0026;
pub const SGTL5000_CHIP_REF_CTRL: u16 = 0x0028;
pub const SGTL5000_CHIP_MIC_CTRL: u16 = 0x002A;
pub const SGTL5000_CHIP_LINE_OUT_CTRL: u16 = 0x002C;
pub const SGTL5000_CHIP_LINE_OUT_VOL: u16 = 0x002E;
pub const SGTL5000_CHIP_ANA_POWER: u16 = 0x0030;
pub const SGTL5000_CHIP_PLL_CTRL: u16 = 0x0032;
pub const SGTL5000_CHIP_CLK_TOP_CTRL: u16 = 0x0034;
pub const SGTL5000_SHIP_ANA_STATUS: u16 = 0x0036;
pub const SGTL5000_CHIP_ANA_TEST1: u16 = 0x0038;
pub const SGTL5000_CHIP_ANA_TEST2: u16 = 0x003A;
pub const SGTL5000_CHIP_SHORT_CTRL: u16 = 0x003C;
pub const SGTL5000_DAP_CONTROL: u16 = 0x0100;
pub const SGTL5000_DAP_PEQ: u16 = 0x0102;
pub const SGTL5000_DAP_BASS_ENHANCE: u16 = 0x0104;
pub const SGTL5000_DAP_BASS_ENHANCE_CTRL: u16 = 0x0106;
pub const SGTL5000_DAP_AUDIO_EQ: u16 = 0x0108;
pub const SGTL5000_DAP_SGTL_SURROUND: u16 = 0x010A;
pub const SGTL5000_DAP_FILTER_COEF_ACCESS: u16 = 0x010C;
pub const SGTL5000_DAP_COEF_WR_B0_MSB: u16 = 0x010E;
pub const SGTL5000_DAP_COEF_WR_B0_LSB: u16 = 0x0110;
pub const SGTL5000_DAP_AUDIO_EQ_BASS_BAND0: u16 = 0x0116;
pub const SGTL5000_DAP_AUDIO_EQ_BAND1: u16 = 0x0118;
pub const SGTL5000_DAP_AUDIO_EQ_BAND2: u16 = 0x011A;
pub const SGTL5000_DAP_AUDIO_EQ_BAND3: u16 = 0x011C;
pub const SGTL5000_DAP_AUDIO_EQ_TREBLE_BAND4: u16 = 0x011E;
pub const SGTL5000_DAP_MAIN_CHAN: u16 = 0x0120;
pub const SGTL5000_DAP_MIX_CHAN: u16 = 0x0122;
pub const SGTL5000_DAP_AVC_CTRL: u16 = 0x0124;
pub const SGTL5000_DAP_AVC_THRESHOLD: u16 = 0x0126;
pub const SGTL5000_DAP_AVC_ATTACK: u16 = 0x0128;
pub const SGTL5000_DAP_AVC_DECAY: u16 = 0x012A;
pub const SGTL5000_DAP_COEF_WR_B1_MSB: u16 = 0x012C;
pub const SGTL5000_DAP_COEF_WR_B1_LSB: u16 = 0x012E;
pub const SGTL5000_DAP_COEF_WR_B2_MSB: u16 = 0x0130;
pub const SGTL5000_DAP_COEF_WR_B2_LSB: u16 = 0x0132;
pub const SGTL5000_DAP_COEF_WR_A1_MSB: u16 = 0x0134;
pub const SGTL5000_DAP_COEF_WR_A1_LSB: u16 = 0x0136;
pub const SGTL5000_DAP_COEF_WR_A2_MSB: u16 = 0x0138;
pub const SGTL5000_DAP_COEF_WR_A2_LSB: u16 = 0x013A;



#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    const SAMPLE_RATE: u32 = 44100;
    const AMOUNT_OF_SAMPLES_BUFFERING: usize = 1100;
    const BUFFER_SIZE: usize = 4 * AMOUNT_OF_SAMPLES_BUFFERING;

    // init CPU
    let mut config = esp_hal::Config::default();
    config.cpu_clock = CpuClock::max();
    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);
    let timer_group_0 = TimerGroup::new(peripherals.TIMG0);
    let timer1: AnyTimer = timer_group_0.timer0.into();
    let timer2: AnyTimer = timer_group_0.timer1.into();
    esp_hal_embassy::init([timer1, timer2]);
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);

    //
    let mut storage = FlashStorage::new();
    

    let mut wt_on_heap = BoxedWavetables::new();
    let mut wt_read_buffer = [0u8; 1200];
    for wt in 0..10 {
        storage.read(0x00220000 + wt * 1200, &mut wt_read_buffer);
        wt_on_heap.add(BoxedWavetable::new(&wt_read_buffer));
    }
    let wt = Rc::new(wt_on_heap);

    let mut sample_read_buffer = [0u8; 1000];


    let mut sample:Vec<u8> = Vec::with_capacity(328 * 1_000);
    for wt in 0..328 {
        let offset = 0x00220000 + 10 * 1200 + wt * 1000;
        let result = storage.read(offset, &mut sample_read_buffer);
        match result {
            Ok(_) => {},
            Err(error) => esp_println::println!("Error {:?}", error),
        }
        let mut buffer = sample_read_buffer.to_vec();
        sample.append(&mut buffer);
    }

    let boxed = BoxedSample::new(sample);
    

    let  sample_on_heap = Rc::new(boxed);

    esp_println::println!("-:= > Heap used {} KB", esp_alloc::HEAP.used()/1000);
  
    let patch = Patch {
        synth_config: SynthConfiguration {
            mode: SynthMode::Mono,
        },
        voices: [
            WaveTableOscillatorConfig {
                soundbank_index: 5,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 0,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
                freq_detune: 1,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 2,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
                freq_detune: 3,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: true,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 4,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
                freq_detune: 5,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 200,
                detune: 0,
                freq: 440,
                freq_detune: 6,
            },
            WaveTableOscillatorConfig {
                soundbank_index: 1,
                glide: false,
                glide_rate: 0,
                detune: 0,
                freq: 440,
                freq_detune: 7,
            },
        ],
        envelops: [
            EnvelopConfiguration {
                attack_time: 10,
                decay_time: 100,
                release_time: 100,
                sustain_level: 80,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
            EnvelopConfiguration {
                attack_time: 50,
                decay_time: 100,
                release_time: 300,
                sustain_level: 90,
            },
        ],
        lfos: [
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 0,
                time: 50,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 1,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 1,
                time: 200,
            },
            WaveTableLoFreqOscillatorConfig {
                soundbank_index: 1,
                time: 200,
            },
        ],
        routering_config: RoutingConfiguration {
            voices_to_envelop: [
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
                VoiceToEnvelopRoute { env: 0 },
            ],
            voice_to_lfo: [
                VoiceToLFORoute {
                    enable: true,
                    voices: [0, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
                VoiceToLFORoute {
                    enable: false,
                    voices: [1, 255],
                },
            ],
            lfo_to_filter: false,
        },
        filter_config: FilterConfig {
            cutoff_frequency: 1_000,
            resonance: 6_000,
            enabled: false,
            kind_of_filter: KindOfFilter::Low,
        },
        mixer_config: MixerConfiguration {
            gain_voices: [5, 0, 0, 0, 0, 0, 0, 0],
            gain_main: 20,
        },
        overdrive_config: OverdriveConfiguration {
            threshold: 1000,
            kind: KindOfOverdrive::Softer,
            enabled: false,
        },
    };
  
    let mut synth: synth::Synth = synth::Synth::new(SAMPLE_RATE as u16, &patch, Rc::clone(&wt), Rc::clone(&sample_on_heap));
    synth.note_on(60, 100);

    //
    
    let mut config = Config::default();
    let mut i2c = I2c::new(
         peripherals.I2C0,
         config,
    ).with_sda(peripherals.GPIO17)
     .with_scl(peripherals.GPIO18);

     let (_, rx_descriptors, tx_buffer, tx_descriptors) = dma_circular_buffers!(0, BUFFER_SIZE);

     let dma = Dma::new(peripherals.DMA);
        let dma_channel = dma.channel0;

        let i2s = I2s::new(
            peripherals.I2S0,
            Standard::Philips,
            DataFormat::Data16Channel16,
            SAMPLE_RATE.Hz(),
            dma_channel.configure(true, DmaPriority::Priority9),
            rx_descriptors,
            tx_descriptors,
        );

       
        let mut i2s_tx = i2s.with_mclk(peripherals.GPIO16)
            .i2s_tx
            .with_bclk(peripherals.GPIO6)
            .with_ws(peripherals.GPIO4)
            .with_dout(peripherals.GPIO5)
            .build();

    // GPIO17 --> 18
    // GPIO18 --> 19

    let mut buffer = [0u8;2];


    let mut result = i2c.write_read(SGTL5000_I2C_ADDR, &[((SGTL5000_CHIP_ID >> 8) & 0xFF) as u8, (SGTL5000_CHIP_ID & 0xFF) as u8], &mut buffer); 
    println!("SGTL500 chip ID: {:#x}, revision: {:#x}, err-code: {:?}",buffer[0], buffer[1], result);
    

    result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_ANA_POWER >> 8) & 0xFF) as u8, (SGTL5000_CHIP_ANA_POWER & 0xFF) as u8, 0x42, 0x60]);
    result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_LINREG_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_LINREG_CTRL & 0xFF) as u8, 0x00, 0x6C]);
    
    // VAG=1.575, normal ramp, +12.5% bias current
    result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_REF_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_REF_CTRL & 0xFF) as u8, 0x01, 0xF2]);

    // LO_VAGCNTRL=1.65V, OUT_CURRENT=0.54mA
    result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_LINE_OUT_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_LINE_OUT_CTRL & 0xFF) as u8, 0x0F, 0x22]);
   result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_SHORT_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_SHORT_CTRL & 0xFF) as u8, 0x11, 0x06]);


   result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_ANA_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_ANA_CTRL & 0xFF) as u8, 0x01, 0x33]);
    result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_ANA_POWER >> 8) & 0xFF) as u8, (SGTL5000_CHIP_ANA_POWER & 0xFF) as u8, 0x6A, 0xFF]);
     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_DIG_POWER >> 8) & 0xFF) as u8, (SGTL5000_CHIP_DIG_POWER & 0xFF) as u8, 0x00, 0x73]);
     Timer::after(Duration::from_millis(400)).await;
     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_LINE_OUT_VOL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_LINE_OUT_VOL & 0xFF) as u8, 0x1D, 0x1D]);

//

     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_CLK_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_CLK_CTRL & 0xFF) as u8, 0x00, 0x04]);
     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_I2S_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_I2S_CTRL & 0xFF) as u8, 0x00, 0x30]);
     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_SSS_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_SSS_CTRL & 0xFF) as u8, 0x00, 0x10]);
     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_ADCDAC_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_ADCDAC_CTRL & 0xFF) as u8, 0x00, 0x00]);
     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_DAC_VOL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_DAC_VOL & 0xFF) as u8, 0x3C, 0x3C]);
     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_ANA_HP_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_ANA_HP_CTRL & 0xFF) as u8, 0x18, 0x18]);
     result = i2c.write(SGTL5000_I2C_ADDR,  &[((SGTL5000_CHIP_ANA_CTRL >> 8) & 0xFF) as u8, (SGTL5000_CHIP_ANA_CTRL & 0xFF) as u8, 0x00, 0x00]);
     
     let text = 0x0036;
     println!("SGTL500 init done, err-code: {:?}", result);
   
     

    println!("Start Synth");

    let mut transfer = i2s_tx.write_dma_circular(tx_buffer).unwrap();
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        match transfer.available() {
            Ok(num_bytes) => {
                let avail = usize::min(BUFFER_SIZE, num_bytes);
                for i in 0..(avail / 4) {
                    let output = synth.clock_and_output();
                    let left = output[0].to_be_bytes();
                    let right = output[1].to_be_bytes();
                    // left
                    buffer[i * 4] = left[1];
                    buffer[i * 4 + 1] = left[0];
                    //right
                    buffer[i * 4 + 2] = right[1];
                    buffer[i * 4 + 3] = right[0];
                }
                let _ = transfer.push(&buffer[0..avail]);
            }
            Err(error) => {
                println!("Error: {:?}", error);
            }
        }
    }

}



