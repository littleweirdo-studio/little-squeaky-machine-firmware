cd firmware
cargo build --release --offline
cd ..
cd soundbank
cargo build
cd ..
espflash write-bin 0x00220000 soundbank/little_squeaky_machine_data_image.bin 
espflash flash --monitor  target/xtensa-esp32s3-none-elf/release/little_squeaky_machine
