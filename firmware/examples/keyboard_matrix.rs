#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]
extern crate alloc;

use alloc::vec;
use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::gpio::Input;
use esp_hal::gpio::InputConfig;
use esp_hal::gpio::Level;
use esp_hal::gpio::Output;
use esp_hal::gpio::OutputConfig;
use esp_hal::gpio::Pull;

use embassy_time::Duration;
use embassy_time::Timer;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::psram;
use esp_hal::rtc_cntl::Rtc;
use esp_println::println;

#[derive(Clone, Copy, Default, Debug)]
struct KeyStatus {
    counter: u8,
    key_down_trigger: bool,
}

#[derive(Debug)]
struct Matrix<T> {
    rows: usize,
    cols: usize,
    data: vec::Vec<vec::Vec<T>>,
}

impl<T> Matrix<T>
where
    T: Copy + Default,
{
    fn new(rows: usize, cols: usize) -> Self {
        let data = vec![vec![T::default(); cols]; rows];
        Matrix { rows, cols, data }
    }

    fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.data.get(row).and_then(|r| r.get(col))
    }

    fn set(&mut self, row: usize, col: usize, value: T) {
        if let Some(r) = self.data.get_mut(row) {
            r[col] = value;
        }
    }
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    // init CPU
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());

    let peripherals = esp_hal::init(config);
    let rtc = Rtc::new(peripherals.LPWR);
    esp_alloc::psram_allocator!(peripherals.PSRAM, psram);
    println!("> Heap size  = {:?} bytes", esp_alloc::HEAP.free());
    println!(
        "> Heap used before allocation of wavetables/synth = {:?} bytes",
        esp_alloc::HEAP.used()
    );

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(timg0.timer0);

    let input_pin_config = InputConfig::default().with_pull(Pull::Down);
    let output_pin_config = OutputConfig::default();

    let mut matrix = Matrix::<KeyStatus>::new(5, 5);

    let mut cols: [Output; 5] = [
        Output::new(peripherals.GPIO4, Level::Low, output_pin_config),
        Output::new(peripherals.GPIO5, Level::Low, output_pin_config),
        Output::new(peripherals.GPIO6, Level::Low, output_pin_config),
        Output::new(peripherals.GPIO7, Level::Low, output_pin_config),
        Output::new(peripherals.GPIO15, Level::Low, output_pin_config),
    ];

    let mut rows: [esp_hal::gpio::Input; 5] = [
        Input::new(peripherals.GPIO14, input_pin_config),
        Input::new(peripherals.GPIO13, input_pin_config),
        Input::new(peripherals.GPIO12, input_pin_config),
        Input::new(peripherals.GPIO11, input_pin_config),
        Input::new(peripherals.GPIO10, input_pin_config),
    ];

  

    loop {
    

        for (col_counter, col) in cols.iter_mut().enumerate() {
            col.set_high();
            for (row_counter, row) in rows.iter().enumerate() {
                let &(mut status) = matrix.get(col_counter, row_counter).unwrap();
                if row.is_high() {
                    if status.counter == 5 {
                        if status.key_down_trigger == false {
                           // println!("key down ({})", col_counter * 5 + row_counter);
                            status.key_down_trigger = true;
                        }
                    } else {
                        status.counter = status.counter + 1;
                    }
                } else {
                    if status.counter > 0 {
                        status.counter = status.counter - 1;
                        if (status.counter == 0) && (status.key_down_trigger == true) {
                            status.key_down_trigger = false;
                          //  println!("key up   ({})", col_counter * 5 + row_counter);
                          //  println!("--");
                        }
                    }
                }
                matrix.set(col_counter, row_counter, status);
            }
            col.set_low();
        }

        Timer::after(Duration::from_millis(5)).await;
    }
}
