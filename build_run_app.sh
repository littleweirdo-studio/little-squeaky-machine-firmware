cargo build --offline --release
espflash flash --monitor  ./target/xtensa-esp32-none-elf/release/little_squeaky-machine
