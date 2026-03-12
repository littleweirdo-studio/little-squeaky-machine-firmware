cd firmware
cargo build --release --offline --message-format short
cd ..
espflash flash --monitor  firmware/target/xtensa-esp32s3-none-elf/release/little_squeaky_machine
