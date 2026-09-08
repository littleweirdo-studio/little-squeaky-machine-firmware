// Add this struct to hold your recording state
struct AudioRecorder {
    buffer: Vec<u8>,
    is_recording: bool,
    sample_rate: u32,
    max_size: usize,  // Maximum buffer size in bytes
}

impl AudioRecorder {
    fn new(sample_rate: u32, max_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(max_size),
            is_recording: false,
            sample_rate,
            max_size,
        }
    }

    fn start_recording(&mut self) {
        self.is_recording = true;
        self.buffer.clear();  // Clear previous recording when starting new one
    }

    fn stop_recording(&mut self) {
        self.is_recording = false;
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }

    fn process_audio(&mut self, data: &[u8]) {
        if self.is_recording {
            // Only add data if there's space left in the buffer
            if self.buffer.len() + data.len() <= self.max_size {
                self.buffer.extend_from_slice(data);
            }
            // Optionally: could implement a circular buffer behavior here
            // by overwriting oldest data when full
        }
    }

    fn get_recording(&self) -> &[u8] {
        &self.buffer
    }

    fn clear_recording(&mut self) {
        self.buffer.clear();
    }
}