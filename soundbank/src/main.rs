use std::io::Read;
use std::{fs::File, io::Write};

use std::fs::{self, read_dir};

use little_weirdo_filesystem::memory_storage::MemoryStorage;
use little_weirdo_filesystem::{BLOCK_SIZE, WeirdoFileSystem};

mod patch;
mod sampler_patch;
mod wav_to_raw;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SoundBank {
    pub name: String,
    pub amount_of_patches: u16,
    pub amount_of_waveforms: u16,
    pub amount_of_samples: u16,
    pub amound_of_sampler_patches: u16,
}

fn main() {
    //generating optimized files for Little Weirdo
    println!("Generate optimized data for Little Weirdo using original sources.\n");
    patch::generate_patches();
    sampler_patch::generate_sampler_patches();
    wav_to_raw::generate_waveforms("data/waveforms/src", "data/waveforms");
    wav_to_raw::generate_waveforms("data/samples/src", "data/samples");

    println!("\nCreate filesystem en store generated optimized data. \n");

    let storage = MemoryStorage::new(0x100000);
    let mut filesystem: WeirdoFileSystem<MemoryStorage> =
        WeirdoFileSystem::new(storage, 0, 0x100000);
    filesystem.format();

    println!(
        "total block ({}) = {} B",
        filesystem.total_blocks(),
        filesystem.total_blocks() * BLOCK_SIZE as u32
    );

    let patches_count = store_items(
        &mut filesystem,
        "data/patches".to_string(),
        "lwp".to_string(),
        100,
    );
    let sampler_patches_count = store_items(
        &mut filesystem,
        "data/sampler_patches".to_string(),
        "lwp".to_string(),
        200,
    );
    let waveforms_count = store_items(
        &mut filesystem,
        "data/waveforms".to_string(),
        "raw".to_string(),
        700,
    );
    let samples_count = store_items(
        &mut filesystem,
        "data/samples".to_string(),
        "raw".to_string(),
        900,
    );

    let soundbank = SoundBank {
        name: "Default LttL Weirdo Soumdbank V0.1".to_string(),
        amount_of_patches: patches_count,
        amount_of_waveforms: waveforms_count,
        amount_of_samples: samples_count,
        amound_of_sampler_patches: sampler_patches_count,
    };

    let mut buf = [0u8; 512];
    let _serialized =
        postcard::to_slice(&soundbank, &mut buf).expect("Failed to serialize to postcard");

    let _ = filesystem.write_key_value(000, &buf);

    println!(
        ":: Image blocks free ({}) = {} B, used ({}) = {} B",
        filesystem.amount_of_free_blocks(),
        filesystem.amount_of_free_blocks() * BLOCK_SIZE as u32,
        filesystem.amount_of_used_blocks(),
        filesystem.amount_of_used_blocks() * BLOCK_SIZE as u32
    );

    let mut buf = [0u8; 256];

    let _ = filesystem.read_key_value(000, &mut buf);
    let patch: SoundBank = postcard::from_bytes(&buf).unwrap();

    println!("name = {}", patch.name);

    // export to disk
    let mut data = filesystem.storage;
    let mut file =
        File::create("little_squeaky_machine_data_image.bin").expect("Unable to create file");
    file.write_all(data.dump()).expect("Unable to write data");
}

fn store_items(
    filesystem: &mut WeirdoFileSystem<MemoryStorage>,
    path: String,
    extension: String,
    key: u16,
) -> u16 {
    // Read directory entries and collect them into a vector
    let mut paths: Vec<_> = read_dir(path).unwrap().filter_map(Result::ok).collect();

    // Sort directory entries by filename for consistent processing
    paths.sort_by_key(|dir| dir.file_name());

    let mut key: u16 = key;
    println!("");
    let mut amount_of_keys: u16 = 0;
    for entry in paths {
        let mut buffer = vec![0u8; 500_000];
        let path = entry.path();
        if path.is_file() {
            // Check if the file has the desired extension
            if let Some(file_extension) = path.extension() {
                if file_extension.to_str().unwrap() == extension {
                    print!(
                        ":: File {:?} stored with key {:?}",
                        path.file_name().unwrap(),
                        key
                    );

                    let mut file = fs::File::open(path).unwrap();
                    let len = file.read(&mut buffer).unwrap();
                    println!(", file size = {:?}", len);
                    let _ = filesystem.write_key_value(key as u32, &buffer[0..len]);
                    key = key + 1;
                    amount_of_keys += 1;
                }
            }
        }
    }
    amount_of_keys
}
