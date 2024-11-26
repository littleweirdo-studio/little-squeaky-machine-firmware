pub fn parse_midi_payload(payload: &[u8]) -> MidiMessage {
    //println!("payload:: {:?} ", payload);
    // if B = 1 then LEN = 12 bits else LEN = 4 bits
    //println!("payload B:: {:?} ", payload[0] & 0b1000_0000 != 0);
    // if J = 1 then a journal is appended to the packet
    //println!("payload J:: {:?} ", payload[0] & 0b0100_0000 != 0);
    // if Z = 1 then a deltatime is provided
    //println!("payload Z:: {:?} ", payload[0] & 0b0010_0000 != 0);
    //println!("payload P:: {:?} ", payload[0] & 0b0001_0000 != 0);
    let b: bool = payload[0] & 0b1000_0000 != 0;
    let len = payload[0] & 0b0000_1111;
    if b {}
    MidiMessage::from(&payload[1..4])
}

/// The MIDI Channel
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
    Ch8,
    Ch9,
    Ch10,
    Ch11,
    Ch12,
    Ch13,
    Ch14,
    Ch15,
    Ch16,
    Invalid,
}

impl Channel {
    /// Convert the MIDI command byte to a Channel
    pub fn from_midi_cmd(v: u8) -> Channel {
        Self::from(v & MIDI_CHANNEL_MASK)
    }
}

impl From<u8> for Channel {
    /// Convert a byte value 0-15 to the Channel.
    /// If you want to extract it from the MIDI event,
    /// use Channel::from_midi_cmd()
    fn from(v: u8) -> Channel {
        use Channel::*;
        match v {
            0 => Ch1,
            1 => Ch2,
            2 => Ch3,
            3 => Ch4,
            4 => Ch5,
            5 => Ch6,
            6 => Ch7,
            7 => Ch8,
            8 => Ch9,
            9 => Ch10,
            10 => Ch11,
            11 => Ch12,
            12 => Ch13,
            13 => Ch14,
            14 => Ch15,
            15 => Ch16,
            _ => Invalid,
        }
    }
}

/// Parameters of a key related event
///
/// This include note on and off and poly key pressure Value is
/// velocity for notes and pressure amount for pressure events.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyEvent {
    /// The note to which this applies
    pub key: MidiNote,
    /// The velocity for notes or pressure value for pressure.
    pub value: u8,
}

/// Parameters for a control change
#[derive(Debug, Eq, PartialEq)]
pub struct ControlEvent {
    /// Control number
    pub control: u8,
    /// Value of the control
    pub value: u8,
}

/// MIDI messages are what is being sent or received on the MIDI system
///
#[derive(Debug, Eq, PartialEq)]
pub enum MidiMessage {
    /// We don't know that message.
    Invalid,
    /// Note on.
    NoteOn(Channel, KeyEvent),
    /// Note off.
    NoteOff(Channel, KeyEvent),
    /// Pressure for notes (aftertouch).
    PolyKeyPressure(Channel, KeyEvent),
    /// Control value changed.
    ControlChange(Channel, ControlEvent),
    /// Program change.
    ProgramChange(Channel, u8),
    /// Channel pressure.
    ChannelPressure(Channel, u8),
    /// Pitch bending. LSB and MSB of the change.
    PitchBend(Channel, u8, u8),
}

pub const SYSEX: u8 = 240;
pub const EVENT_TYPE_MASK: u8 = 0b1111_0000;
pub const MIDI_CHANNEL_MASK: u8 = 0b0000_1111;
pub const NOTE_OFF: u8 = 0x80;
pub const NOTE_ON: u8 = 0x90;
pub type MidiNote = u8;
impl From<&[u8]> for MidiMessage {
    /// Create a MidiMessage from raw data as received from the MIDI driver.
    fn from(data: &[u8]) -> MidiMessage {
        if data.len() < 3 {
            MidiMessage::Invalid
        } else {
            let (event, channel) = if data[0] < SYSEX {
                (data[0] & EVENT_TYPE_MASK, data[0] & MIDI_CHANNEL_MASK)
            } else {
                (data[0], 0u8)
            };
            match event {
                NOTE_OFF => {
                    // Note Off
                    MidiMessage::NoteOff(
                        Channel::from(channel),
                        KeyEvent {
                            key: data[1],
                            value: data[2],
                        },
                    )
                }
                NOTE_ON => {
                    // Note On
                    MidiMessage::NoteOn(
                        Channel::from(channel),
                        KeyEvent {
                            key: data[1],
                            value: data[2],
                        },
                    )
                }
                /*     POLYPHONIC_KEY_PRESSURE => MidiMessage::PolyKeyPressure(
                    Channel::from(channel),
                    KeyEvent {
                        key: data[1],
                        value: data[2],
                    },
                ),
                CONTROL_CHANGE => MidiMessage::ControlChange(
                    Channel::from(channel),
                    ControlEvent {
                        control: data[1],
                        value: data[2],
                    },
                ),
                consts::PROGRAM_CHANGE => {
                    MidiMessage::ProgramChange(Channel::from(channel), data[1])
                }
                consts::CHANNEL_KEY_PRESSURE => {
                    MidiMessage::ChannelPressure(Channel::from(channel), data[1])
                }
                consts::PITCH_BEND_CHANGE => {
                    MidiMessage::PitchBend(Channel::from(channel), data[1], data[2])
                }
                consts::SYSEX => MidiMessage::sysex_message_from(data),
                // TODO handle other system message
                */
                _ => MidiMessage::Invalid,
            }
        }
    }
}
