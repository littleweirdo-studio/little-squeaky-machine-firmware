#![no_std]
#![no_main]
use esp32_hal::entry;
use esp_backtrace as _;
use esp_println::println;



pub trait Midi {
    fn test(&self);
}

pub trait Sweet {
    fn test(&self);
}

impl Midi for Test {
    fn test(&self) {
        println!("Test:test");
    }
}

pub struct Test {

}


#[entry]
fn main() -> ! {
    println!("trait test");
    let help = Test {};
    hello(help);
    loop {

    }
}

fn hello<T: Midi>(t: T) {
    t.test()
}