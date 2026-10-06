use alloc::vec;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    peripherals::Peripherals,
};
use little_weirdo_gui::{Event, MAPPING_SIGNAL};
extern crate alloc;

#[derive(Clone, Copy, Default, Debug)]
pub struct KeyStatus {
    pub counter: u8,
    pub key_down_trigger: bool,
}

#[derive(Debug)]
struct Matrix<T> {
    data: vec::Vec<vec::Vec<T>>,
}

impl<T> Matrix<T>
where
    T: Copy + Default,
{
    fn new(rows: usize, cols: usize) -> Self {
        let data = vec![vec![T::default(); cols]; rows];
        Matrix { data }
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

pub const KEYPAD_EVENT_DOWN: u8 = 40;
pub const KEYPAD_EVENT_UP: u8 = 41;
pub const GPIO0_MENU_BUTTON_PRESSED: u8 = 39;
const DEBUG: bool = false;
const GPIO0_DEBOUNCE_SAMPLES: u8 = 5;

#[embassy_executor::task]
pub async fn keyboard_input_task() {
    let mut cols: [Output; 5];
    let rows: [esp_hal::gpio::Input; 5];
    let gpio0_button: Input;

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let input_pin_config = InputConfig::default().with_pull(Pull::Down);
        let gpio0_button_config = InputConfig::default().with_pull(Pull::Up);
        let output_pin_config = OutputConfig::default();

        cols = [
            Output::new(peripherals.GPIO14, Level::Low, output_pin_config),
            Output::new(peripherals.GPIO13, Level::Low, output_pin_config),
            Output::new(peripherals.GPIO12, Level::Low, output_pin_config),
            Output::new(peripherals.GPIO11, Level::Low, output_pin_config),
            Output::new(peripherals.GPIO10, Level::Low, output_pin_config),
        ];

        rows = [
            Input::new(peripherals.GPIO4, input_pin_config),
            Input::new(peripherals.GPIO5, input_pin_config),
            Input::new(peripherals.GPIO6, input_pin_config),
            Input::new(peripherals.GPIO7, input_pin_config),
            Input::new(peripherals.GPIO15, input_pin_config),
        ];
        gpio0_button = Input::new(peripherals.GPIO0, gpio0_button_config);
    }

    let mut matrix = Matrix::<KeyStatus>::new(5, 5);
    let mut gpio0_pressed_samples = 0u8;
    let mut gpio0_button_pressed = false;
    loop {
        if gpio0_button.is_low() {
            gpio0_pressed_samples = gpio0_pressed_samples.saturating_add(1);
            if gpio0_pressed_samples >= GPIO0_DEBOUNCE_SAMPLES && !gpio0_button_pressed {
                gpio0_button_pressed = true;
                if MAPPING_SIGNAL
                    .try_send(Event {
                        event_identifier: GPIO0_MENU_BUTTON_PRESSED,
                        id: 0,
                        value: 0,
                    })
                    .is_err()
                {
                    esp_println::println!("GPIO0 menu button event dropped: mapping queue full");
                }
            }
        } else {
            gpio0_pressed_samples = 0;
            gpio0_button_pressed = false;
        }

        for (col_counter, col) in cols.iter_mut().enumerate() {
            Timer::after(Duration::from_millis(1)).await;
            col.set_high();
            for (row_counter, row) in rows.iter().enumerate() {
                let &(mut status) = matrix.get(col_counter, row_counter).unwrap();
                if row.is_high() {
                    if status.counter == 5 {
                        if !status.key_down_trigger {
                            if DEBUG {
                                esp_println::println!(
                                    "key down ({})",
                                    col_counter * 5 + row_counter
                                )
                            };
                            let event = Event {
                                event_identifier: KEYPAD_EVENT_DOWN,
                                id: (col_counter * 5 + row_counter) as u8,
                                value: 0,
                            };
                            let _ = MAPPING_SIGNAL
                                .try_send(event);
                            status.key_down_trigger = true;
                        }
                    } else {
                        status.counter += 1;
                    }
                } else {
                    if status.counter > 0 {
                        status.counter -= 1;
                        if (status.counter == 0) && (status.key_down_trigger) {
                            if DEBUG {
                                esp_println::println!(
                                    "key up   ({})",
                                    col_counter * 5 + row_counter
                                );
                                esp_println::println!("--");
                            }
                            status.key_down_trigger = false;
                            let event = Event {
                                event_identifier: KEYPAD_EVENT_UP,
                                id: (col_counter * 5 + row_counter) as u8,
                                value: 0,
                            };
                            let _ = MAPPING_SIGNAL
                                .try_send(event.clone());
                        }
                    }
                }
                matrix.set(col_counter, row_counter, status);
            }
            col.set_low();
        }

        Timer::after(Duration::from_millis(1)).await;
    }
}
