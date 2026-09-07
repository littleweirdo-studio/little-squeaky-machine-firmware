cd experiment
cargo build --release --offline
cd ..
espflash flash --monitor  ./experiment/target/xtensa-esp32s3-none-elf/release/experiment
