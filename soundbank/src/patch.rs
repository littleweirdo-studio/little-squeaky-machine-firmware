use little_weirdo::synth::patch::Patch;
use std::fs::{self, File, read_dir};
use std::io::Write;
use std::path::Path;

pub fn generate_patches() {
      println!(":: Generate patches.");
   

    // Load a synth patch from a JSON file
    let patch_dir = "data/patches/src";
    let patch_dir_dst = "data/patches";

    // Read all files from the patches directory
    let mut paths: Vec<_> = read_dir(patch_dir)
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    // Sort directory entries by filename for consistent processing
    paths.sort_by_key(|dir| dir.file_name());

    for entry in paths {
        let path = entry.path();

        // Only process JSON files
        if let Some(extension) = path.extension() {
            if extension == "json" {
                // Read the JSON file
                let json_content = fs::read_to_string(&path).expect("Failed to read JSON file");

                // Parse JSON
                let patch: Patch = serde_json::from_str(&json_content).unwrap();
                // Serialize to postcard format
                let mut buf = [0u8; 512];
                let serialized =
                    postcard::to_slice(&patch, &mut buf).expect("Failed to serialize to postcard");

                // Create output filename (same name, .lwp extension, in parent directory)
                let output_path = Path::new(patch_dir_dst)
                    .join(path.file_stem().unwrap()) // Get filename without extension
                    .with_extension("lwp"); // Add .lwp extension

                // Write to file
                let mut postcard_patch_file =
                    File::create(&output_path).expect("Failed to create output file");
                postcard_patch_file
                    .write_all(&serialized)
                    .expect("Failed to write to file");

                //println!("Converted {} to {}", path.display(), output_path.display());
            }
        }
    }
}
