use std::io::Read;
use std::{fs::File, io::Write};

use std::fs::{self, read_dir};

use little_weirdo_filesystem::WeirdoFileSystem;
use little_weirdo_filesystem::memory_storage::MemoryStorage;

mod patch;
mod wav_to_raw;

fn main() {
    //generating optimized files for Little Weirdo
    println!("Generate optimized data for Little Weirdo using original sources.\n");
     patch::generate_patches();
     wav_to_raw::generate_waveforms("data/waveforms/src","data/waveforms");
    wav_to_raw::generate_waveforms("data/samples/src","data/samples");
    

    println!("\nCreate filesystem en store generated optimized data. \n");

    let storage = MemoryStorage::new(0x100000);
    let mut filesystem: WeirdoFileSystem<MemoryStorage> =
        WeirdoFileSystem::new(storage, 0, 0x100000);
    filesystem.format();

    store_items(&mut filesystem, "data/patches".to_string(), "lwp".to_string(), 100);
    store_items(&mut filesystem, "data/waveforms".to_string(), "raw".to_string(),  700);
    store_items(&mut filesystem, "data/samples".to_string(), "raw".to_string(), 900);

    // export to disk
    let mut data = filesystem.storage;
    let mut file =
        File::create("little_squeaky_machine_data_image.bin").expect("Unable to create file");
    file.write_all(data.dump()).expect("Unable to write data");
    
}

fn store_items(filesystem: &mut WeirdoFileSystem<MemoryStorage>, path: String, extension: String, key: u16) {
    // Read directory entries and collect them into a vector
    let mut paths: Vec<_> = read_dir(path).unwrap().filter_map(Result::ok).collect();

    // Sort directory entries by filename for consistent processing
    paths.sort_by_key(|dir| dir.file_name());

    let mut key: u16 = key;
    for entry in paths {
        let mut buffer: [u8; 32000] = [0; 32000];
        let path = entry.path();
        if path.is_file() {
            // Check if the file has the desired extension
            if let Some(file_extension) = path.extension() {
                if file_extension.to_str().unwrap() == extension {
                    println!("file {:?} stored with key {:?}", path.file_name(), key);

                    let mut file = fs::File::open(path).unwrap();
                    let len = file.read(&mut buffer).unwrap();
                    println!("file size = {:?}", len);
                    filesystem.write_key_value(key, &buffer[0..len]);
                    key = key + 1;
                }
            }
        }
    }
}
