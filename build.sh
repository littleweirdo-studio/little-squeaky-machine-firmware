cargo build --release
cargo build --example on_device_synth_benchmark --release

espflash flash --monitor  ./target/xtensa-esp32-none-elf/release/little_squeaky-machine
