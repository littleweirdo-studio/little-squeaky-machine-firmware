use alloc::vec;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    peripherals::Peripherals,
};
use little_weirdo_gui::{EventType, KeyEvent, MAPPING_SIGNAL};
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



#[embassy_executor::task]
pub async fn keyboard_input_task() {
    let mut cols: [Output; 5];
    let rows: [esp_hal::gpio::Input; 5];

    let keymap_down: [KeyEvent; 25] = [
        // 0 -- 9
        KeyEvent { event: EventType::KeypadDown, id: 0, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 1, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 2, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 3, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 4, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 5, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 6, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 7, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 8, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 9, value: 0 },
        // 10 -- 19
        KeyEvent { event: EventType::KeypadDown, id: 10, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 11, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 12, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 13, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 14, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 15, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 16, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 17, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 18, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 19, value: 0 },
        // 20 -- 24
        KeyEvent { event: EventType::KeypadDown, id: 20, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 21, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 22, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 23, value: 0 },
        KeyEvent { event: EventType::KeypadDown, id: 24, value: 0 },
    ];

    let keymap_up: [KeyEvent; 25] = [
          // 0 -- 9
        KeyEvent { event: EventType::KeypadUp, id: 0, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 1, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 2, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 3, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 4, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 5, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 6, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 7, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 8, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 9, value: 0 },
        // 10 -- 19
        KeyEvent { event: EventType::KeypadUp, id: 10, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 11, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 12, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 13, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 14, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 15, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 16, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 17, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 18, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 19, value: 0 },
        // 20 -- 24
        KeyEvent { event: EventType::KeypadUp, id: 20, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 21, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 22, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 23, value: 0 },
        KeyEvent { event: EventType::KeypadUp, id: 24, value: 0 },
    ];

    unsafe {
        let peripherals: Peripherals = Peripherals::steal();
        let input_pin_config = InputConfig::default().with_pull(Pull::Down);
        let output_pin_config = OutputConfig::default();

        cols = [
            Output::new(peripherals.GPIO4, Level::Low, output_pin_config),
            Output::new(peripherals.GPIO5, Level::Low, output_pin_config),
            Output::new(peripherals.GPIO6, Level::Low, output_pin_config),
            Output::new(peripherals.GPIO7, Level::Low, output_pin_config),
            Output::new(peripherals.GPIO15, Level::Low, output_pin_config),
        ];

        rows = [
            Input::new(peripherals.GPIO14, input_pin_config),
            Input::new(peripherals.GPIO13, input_pin_config),
            Input::new(peripherals.GPIO12, input_pin_config),
            Input::new(peripherals.GPIO11, input_pin_config),
            Input::new(peripherals.GPIO10, input_pin_config),
        ];
    }

    let mut matrix = Matrix::<KeyStatus>::new(5, 5);

    loop {
        for (col_counter, col) in cols.iter_mut().enumerate() {
            Timer::after(Duration::from_millis(1)).await;
            col.set_high();
            for (row_counter, row) in rows.iter().enumerate() {
                let &(mut status) = matrix.get(col_counter, row_counter).unwrap();
                if row.is_high() {
                    if status.counter == 5 {
                        if status.key_down_trigger == false {
                          // println!("key down ({})", col_counter * 5 + row_counter);
                            let _ = MAPPING_SIGNAL.try_send(keymap_down[col_counter * 5 + row_counter].clone());
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
                            let _ = MAPPING_SIGNAL.try_send(keymap_up[col_counter * 5 + row_counter].clone());
                      //      println!("key up   ({})", col_counter * 5 + row_counter);
                      //      println!("--");
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
