cd soundbank
cargo build
cd ..
./target/debug/soundbank 
espflash write-bin 0x00220000 soundbank/little_squeaky_machine_data_image.bin 