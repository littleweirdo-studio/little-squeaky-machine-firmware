// Import necessary standard library components
use std::{
    fs::{File, read_dir},
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

// Import the Hound crate for WAV file handling
use hound::WavReader;
use tera::{Context, Tera};

// Main function - entry point of the program
pub fn generate_waveforms(source: &str, destination: &str) {
    println!(":: Generate waveforms.");
    // Get the original soundbank source path
    let soundbank_original_source_path = Path::new(source);
    // Get the output soundbank source path
    let soundbank_source_path = Path::new(destination);

    // Initialize a counter for tracking processed files
    let mut counter = 0;

    // Read directory entries and collect them into a vector
    let mut paths: Vec<_> = read_dir(soundbank_original_source_path)
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    // Sort directory entries by filename for consistent processing
    paths.sort_by_key(|dir| dir.file_name());

    // Get current timestamp for documentation
    let now = SystemTime::now();
    let _since_unix_epoch = now.duration_since(UNIX_EPOCH).unwrap();

    let tera = Tera::new("templates/**/*").expect("Failed to load templates");
    let mut context = Context::new();
    let mut entries = Vec::new();

    let soundbank_reference_file_name = Path::new(soundbank_source_path).join("README.md");

    // Process each file in the directory
    for entry in paths {
        println!(":: Processing file {:?}", entry.path());
        // Open the WAV file for reading
        let mut open = WavReader::open(entry.path()).unwrap();
        if open.spec().channels == 1
            && open.spec().bits_per_sample == 16
            && open.spec().sample_rate == 44100
        {
            entries.push(serde_json::json!({
                "index": counter,
                "original": entry.file_name().to_string_lossy(),
                "size":  open.samples::<i16>().len(),
            }));

            // Create the output filename for the raw audio data
            let soundbank_source_file_name = format!("{:03}_sample.raw", counter);
            let soundbank_source =
                Path::new(soundbank_source_path).join(soundbank_source_file_name);

            // Create the output file for raw audio data
            let mut soundbank_source_file =
                File::create(soundbank_source).expect("Failed to open file");

            // Iterate through all samples in the WAV file
            for sample in open.samples::<i16>().into_iter() {
                // Extract the sample value
                let output = sample.unwrap();

                // Convert the sample to little-endian byte representation and write to output file
                let _ = soundbank_source_file.write(&output.to_ne_bytes());
            }
            counter = counter + 1;
        } else {
            println!("not the right format {:?}", entry.path());
            println!("spec = {:?}", open.spec());
        }
    }

    context.insert("entries", &entries);
    let total_size: i64 = entries
        .iter()
        .map(|e| e["size"].as_i64().unwrap_or(0))
        .sum();
    context.insert("total_size", &((total_size * 2) / 1000));

    let rendered = tera
        .render("README.md.tera", &context)
        .expect("Failed to render template");

    let mut readme_file = File::create(soundbank_reference_file_name).expect("Failed to open file");
    let _ = readme_file.write(rendered.as_bytes());
}
