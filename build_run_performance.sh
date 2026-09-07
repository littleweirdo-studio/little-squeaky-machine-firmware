cd firmware_performance
cargo build --release --offline
cd ..
espflash flash --monitor  ./firmware_performance/target/xtensa-esp32s3-none-elf/release/little_squeaky_machine_performance
