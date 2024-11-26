//     ___    _______  _______  ___        _     _
//    |   |  |_     _||_     _||   | ____ | ||.|| |
//    |   |    |   |    |   |  |   ||____||       |
//    |_____|  |___|    |___|  |_____|    |__| |__|
//
//   (c) 2024
//

#![no_std]
#![no_main]
#[warn(unused_imports)]

use esp32_hal as hal;
use esp_hal_common::{prelude::nb::block, uart, Rtc};
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Uart};
pub type BootButton = crate::hal::gpio::Gpio0<crate::hal::gpio::Input<crate::hal::gpio::PullDown>>;
use esp_backtrace as _;
use esp_println::println;

#[entry]
fn main() -> ! {
    // setup the system devices and clocking (MAX speed)
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut uart0 = Uart::new(peripherals.UART0, &clocks);

    let _ = uart0.write_bytes("@littleweirdo > ".as_bytes());
    let mut input_buffer: [u8; 128] = [0u8; 128];
    let mut input_buffer_pointer = 0;

    loop {
        let read = block!(uart0.read());
        match read {
            Ok(input) => match input {
                0x0D => {
                    if "hello".as_bytes() == &input_buffer[0..input_buffer_pointer] {
                        uart0.write_bytes("\nhello world!".as_bytes());
                    } else if "time".as_bytes() == &input_buffer[0..input_buffer_pointer] {
                        uart0.write_bytes("\ntime {}".as_bytes());
                    } else {
                        uart0.write_bytes("\ncommand \u{001b}[31m[".as_bytes());
                        uart0.write_bytes(&input_buffer[0..input_buffer_pointer]);
                        uart0.write_bytes("]\u{001b}[0m not found".as_bytes());
                    }
                    uart0.write_bytes("\n@littleweirdo > ".as_bytes());
                    input_buffer_pointer = 0;
                }
                0x20...0x7F => {
                    //echo and store potential command
                    uart0.write(input);
                    input_buffer[input_buffer_pointer] = input;
                    input_buffer_pointer += 1;
                }
                // backspace
                0x08 => {
                    if input_buffer_pointer > 0 {
                        uart0.write(input);
                        uart0.write(0x20);
                        uart0.write(input);

                        input_buffer_pointer -= 1
                    };
                }
                _ => {
                    // ignore special chars
                }
            },
            Err(error) => {
                println!("{:?}", error)
            }
        };
    }
}


