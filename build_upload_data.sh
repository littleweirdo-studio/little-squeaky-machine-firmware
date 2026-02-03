cd soundbank
cargo run
cd ..
espflash write-bin 0x00220000 soundbank/little_squeaky_machine_data_image.bin 