cd firmware_performance
cargo build --release --offline
cd ..
espflash flash --monitor  ./target/xtensa-esp32-none-elf/release/little-squeaky-machine-performance
