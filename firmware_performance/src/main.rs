#![no_std]
#![no_main]

use alloc::{string::String, sync::Arc, vec};

use embassy_executor::Spawner;
use esp_alloc::heap_allocator;
use esp_backtrace as _;
use esp_hal::rtc_cntl::Rtc;
use esp_hal::{peripherals::Peripherals, ram};
use esp_println::println;
use esp_storage::FlashStorage;
use little_weirdo::synth;
use little_weirdo::synth::patch::Patch;
use little_weirdo::synth::{
    Synth,
    data::{
        patches::{BoxedPatch, BoxedPatches},
        waveforms::{BoxedWaveform, BoxedWaveforms},
    },
};
use little_weirdo_filesystem::WeirdoFileSystem;

use esp_hal::timer::timg::TimerGroup;

extern crate alloc;
use esp_alloc as _;
use esp_backtrace as _;

use serde::Deserialize;
use serde::Serialize;
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SoundBank {
    pub name: String,
    pub amount_of_patches: u16,
    pub amount_of_waveforms: u16,
    pub amount_of_samples: u16,
    pub amound_of_sampler_patches: u16,
}

const SAMPLE_RATE: u32 = 44_100;
const DELAY_US: u32 = 1_000_000 / SAMPLE_RATE;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    // init CPU
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());

    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);

    // Use 64kB in dram2_seg for the heap, which is otherwise unused.
    heap_allocator!(#[ram(reclaimed)] size: 64000);

    // Initialize PSRAM and add it as a heap memory region
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let software_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(timg0.timer0, software_interrupt.software_interrupt0);

    println!("> performance run start");
    println!("> Heap size  = {:?} bytes", esp_alloc::HEAP.free());
    println!(
        "> Heap used before allocation of wavetables/synth = {:?} bytes",
        esp_alloc::HEAP.used()
    );

    println!("> performance run stop");

    let storage;
    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        storage = FlashStorage::new(peripherals.FLASH);
    }

    let mut filesystem: WeirdoFileSystem<FlashStorage> =
        WeirdoFileSystem::new(storage, 0x00220000, 0x100000);

    let mut buf = vec![0u8; 256];

    let _ = filesystem.read_key_value(000, buf.as_mut_slice());
    let sound_bank: SoundBank = postcard::from_bytes(&buf).unwrap();

    let mut wt_on_heap = BoxedWaveforms::new();
    let mut wt_read_buffer = alloc::vec![0u8; 1200];
    for wt in 0..10 {
        let _ = filesystem
            .read_key_value(700 + wt, &mut wt_read_buffer)
            .unwrap();
        wt_on_heap.add(BoxedWaveform::new(&wt_read_buffer));
    }
    let wt = Arc::new(wt_on_heap);

    esp_println::println!("-:= > \n{}-:= >", esp_alloc::HEAP.stats());

    let mut patches_on_heap = BoxedPatches::new();
    let mut patch_read_buffer = vec![0u8; 512];
    for pt in 0..sound_bank.amount_of_patches as u32 {
        let length_of_patch = filesystem
            .read_key_value(100 + pt, &mut patch_read_buffer)
            .unwrap();
        let patch: Patch =
            postcard::from_bytes(&patch_read_buffer[..length_of_patch as usize]).unwrap();
        println!("patch {}", patch.name);
        patches_on_heap.add(BoxedPatch::new(patch));
    }

    let patches = Arc::new(patches_on_heap);

    let mut synth: synth::Synth =
        Synth::new(SAMPLE_RATE as u16, 16, Arc::clone(&patches), Arc::clone(&wt));

    println!(
        "> Heap used after allocation of wavetables/synth = {:?} bytes",
        esp_alloc::HEAP.used()
    );

    let mut sum = 0;
    let mut overrun = 0;
    let mut high = 0;
    let mut low: u64 = DELAY_US as u64;
    let mut moment: u32 = 0;
    for _x in 0..20 {
        let start_time = rtc.current_time_us();
        synth.load_patch(0);
        let stop_time = rtc.current_time_us();
        esp_println::println!(
            "> patch change in {} µs (max {}µs)",
            stop_time - start_time,
            DELAY_US
        );
    }

    for _x in 0..10 {
        synth.note_on(60, 100);
        synth.note_on(61, 100);

        synth.note_on(62, 100);
        synth.note_on(63, 100);
        synth.note_on(64, 100);
        synth.note_on(65, 100);

        for n in 0..SAMPLE_RATE {
            let start_time = rtc.current_time_us();
            let _output = synth.clock_and_output();
            let stop_time = rtc.current_time_us();
            let calculation_cost = stop_time - start_time;
            sum += calculation_cost;
            if calculation_cost < DELAY_US as u64 {
                if low > calculation_cost {
                    low = calculation_cost
                };
            } else {
                overrun += 1;
                //  esp_println::println!("> ! highest process time {}µs (@{})", calculation_cost, n);

                if high < calculation_cost {
                    high = calculation_cost;
                    moment = n;
                };
            }
        }

        synth.note_off(60);
        synth.note_off(61);
        synth.note_off(62);
        synth.note_off(63);
        synth.note_off(64);
        synth.note_off(65);

        for _n in 0..SAMPLE_RATE {
            synth.clock_and_output();
        }
        esp_println::println!("----- 1 sec cycle @ 44.1KHz (max 22µs ) -----");
        esp_println::println!("> average - total compute time spend  {}", sum);
        esp_println::println!("> clock :: average {}µs", sum / SAMPLE_RATE as u64);
        esp_println::println!("> deadline passed {}x,", overrun);
        esp_println::println!("> highest process time {}µs (@{})", high, moment);
        esp_println::println!("> lowest  process time {}µs", low);
        esp_println::println!("> headroom = {} %", (1_000_000 - sum) / 10000);
        esp_println::println!("----------------------------------------------");
        sum = 0;
        overrun = 0;
        high = 0;
        low = DELAY_US as u64;
    }
    println!("> performance run stop");
}
