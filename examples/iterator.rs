#![no_std]

struct MidiMessages {
    count: usize,
}
#[derive(PartialEq, Debug)]
enum MidiMessage {
    NoteOn,
    NoteOff,
}

impl MidiMessages {
    fn new() -> MidiMessages {
        MidiMessages { count: 0 }
    }
}

// Then, we implement `Iterator` for our `Counter`:

impl Iterator for MidiMessages {
    // we will be counting with usize
    type Item = MidiMessage;

    // next() is the only required method
    fn next(&mut self) -> Option<MidiMessage> {
        // Increment our count. This is why we started at zero.
        self.count += 1;

        // Check to see if we've finished counting or not.
        if self.count < 6 {
            Some(MidiMessage::NoteOn)
        } else {
            None
        }
    }
}

fn main() {
    let midi_messages = MidiMessages::new();
    for x in midi_messages {
        println!("{:?}", x);
    }
}
