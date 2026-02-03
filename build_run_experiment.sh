cd experiment
cargo build --release --offline
cd ..
espflash flash --monitor  ./target/xtensa-esp32-none-elf/release/experiment
