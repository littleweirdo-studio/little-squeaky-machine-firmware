pub const RPT_MIDI_PAYLOAD: u8 = 0x61u8;
const RPT_MIDI_MESSAGE_OK: u16 = 0x4F4B;
const RPT_MIDI_MESSAGE_IN: u16 = 0x494E;
const RPT_MIDI_MESSAGE_BY: u16 = 0x4259;
const RPT_MIDI_MESSAGE_CK: u16 = 0x434B;
/// Wrapper around a byte-slice of RTP data, providing accessor methods for the RTP header fields.
pub struct AppleMIDIReader<'a> {
    buf: &'a [u8],
}

/// Reasons for `RtpHeader::new()` to fail
#[derive(Debug)]
pub enum AppleMIDIReaderError {
    /// Buffer too short to be valid RTP packet
    BufferTooShort(usize),
    MagicNumberMismatch,
}

#[derive(Debug)]
pub enum AppleMIDICommand {
    IN,
    BY,
    OK,
    NO,
    CK,
    XX,
}

impl<'a> AppleMIDIReader<'a> {
    const MIN_HEADER_LEN: usize = 4;

    pub fn new(b: &'a [u8]) -> Result<AppleMIDIReader<'_>, AppleMIDIReaderError> {
        if b.len() < Self::MIN_HEADER_LEN {
            return Err(AppleMIDIReaderError::BufferTooShort(b.len()));
        }
        if !(b[0] & 0xff == 0xff && b[1] & 0xff == 0xff) {
            return Err(AppleMIDIReaderError::MagicNumberMismatch);
        }
        let r = AppleMIDIReader { buf: b };
        Ok(r)
    }

    pub fn command(&self) -> AppleMIDICommand {
        let command_received = u16::from_be_bytes([self.buf[2], self.buf[3]]);
        //    println!("{:#01x}", command_received);
        match command_received {
            RPT_MIDI_MESSAGE_IN => {
                // IN
                AppleMIDICommand::IN
            }
            RPT_MIDI_MESSAGE_BY => {
                // BY
                AppleMIDICommand::BY
            }
            RPT_MIDI_MESSAGE_CK => {
                // CK
                AppleMIDICommand::CK
            }
            _ => AppleMIDICommand::XX,
        }
    }

    pub fn protocol_version(&self) -> u32 {
        u32::from_be_bytes([self.buf[4], self.buf[5], self.buf[6], self.buf[7]])
    }
    pub fn initiator_token(&self) -> u32 {
        u32::from_be_bytes([self.buf[8], self.buf[9], self.buf[10], self.buf[11]])
    }
    pub fn SSRC(&self) -> u32 {
        match self.command() {
            AppleMIDICommand::BY | AppleMIDICommand::IN => {
                u32::from_be_bytes([self.buf[12], self.buf[13], self.buf[14], self.buf[15]])
            }
            AppleMIDICommand::CK => {
                u32::from_be_bytes([self.buf[5], self.buf[6], self.buf[7], self.buf[8]])
            }
            _ => 0,
        }
    }

    pub fn amount_of_timestamps(&self) -> u8 {
        self.buf[8] + 1
    }

    pub fn get_timestamp(&self, index: u8) -> u64 {
        u64::from_be_bytes([
            self.buf[16 + index as usize * 8],
            self.buf[16 + index as usize * 8 + 1],
            self.buf[16 + index as usize * 8 + 2],
            self.buf[16 + index as usize * 8 + 3],
            self.buf[16 + index as usize * 8 + 4],
            self.buf[16 + index as usize * 8 + 5],
            self.buf[16 + index as usize * 8 + 6],
            self.buf[16 + index as usize * 8 + 7],
        ])
    }

    pub fn createPacket(
        &self,
        command: AppleMIDICommand,
        own_ssrc: u32,
        initiator_token: u32,
        buffer: &mut [u8],
        received_timestamp: u64,
    ) -> usize {
        // magic number
        buffer[0] = 0xFFu8;
        buffer[1] = 0xFFu8;

        match command {
            AppleMIDICommand::OK => {
                // version number (2)
                buffer[4] = 0x00u8;
                buffer[5] = 0x00u8;
                buffer[6] = 0x00u8;
                buffer[7] = 0x02u8;

                // initiator_TOKEN
                let mut bytes = initiator_token.to_be_bytes();
                buffer[8] = bytes[0];
                buffer[9] = bytes[1];
                buffer[10] = bytes[2];
                buffer[11] = bytes[3];

                // ssrc_TOKEN
                bytes = own_ssrc.to_be_bytes();
                buffer[12] = bytes[0];
                buffer[13] = bytes[1];
                buffer[14] = bytes[2];
                buffer[15] = bytes[3];

                // name
                buffer[16] = 0x4Fu8;
                buffer[17] = 0x4Fu8;
                buffer[18] = 0x4Fu8;
                buffer[19] = 0x4Fu8;
            }
            AppleMIDICommand::CK => {
                // ssrc_TOKEN
                let bytes = own_ssrc.to_be_bytes();
                buffer[4] = bytes[0];
                buffer[5] = bytes[1];
                buffer[6] = bytes[2];
                buffer[7] = bytes[3];
                // count
                buffer[8] = 0x01;
                // unused
                buffer[9] = 0x00;
                buffer[10] = 0x00;
                buffer[11] = 0x00;
                // received timestamp
                let timestamp = received_timestamp.to_be_bytes();
                buffer[12] = timestamp[0];
                buffer[13] = timestamp[1];
                buffer[14] = timestamp[2];
                buffer[15] = timestamp[3];
                buffer[16] = timestamp[4];
                buffer[17] = timestamp[5];
                buffer[18] = timestamp[6];
                buffer[19] = timestamp[7];
                // current time
                let now: [u8; 8] = 0x34233478u64.to_be_bytes();
                buffer[20] = now[0];
                buffer[21] = now[1];
                buffer[22] = now[2];
                buffer[23] = now[3];
                buffer[24] = now[4];
                buffer[25] = now[5];
                buffer[26] = now[6];
                buffer[27] = now[7];
                //
                buffer[28] = 0x00u8;
                buffer[29] = 0x00u8;
                buffer[30] = 0x00u8;
                buffer[31] = 0x00u8;
                buffer[32] = 0x00u8;
                buffer[33] = 0x00u8;
                buffer[34] = 0x00u8;
                buffer[35] = 0x00u8;
            }
            _ => {
                buffer[2] = 0x00u8;
                buffer[3] = 0x00u8;
            }
        }

        match command {
            AppleMIDICommand::OK => {
                buffer[2] = RPT_MIDI_MESSAGE_OK.to_be_bytes()[0];
                buffer[3] = RPT_MIDI_MESSAGE_OK.to_be_bytes()[1];
                return 20;
            }
            AppleMIDICommand::CK => {
                buffer[2] = RPT_MIDI_MESSAGE_CK.to_be_bytes()[0];
                buffer[3] = RPT_MIDI_MESSAGE_CK.to_be_bytes()[1];
                return 36;
            }
            _ => {}
        }
        0
    }
}

/// Wrapper around a byte-slice of RTP data, providing accessor methods for the RTP header fields.
pub struct RtpReader<'a> {
    buf: &'a [u8],
}

/// Reasons for `RtpHeader::new()` to fail
#[derive(Debug)]
pub enum RtpReaderError {
    /// Buffer too short to be valid RTP packet
    BufferTooShort(usize),
    /// Only RTP version 2 supported
    UnsupportedVersion(u8),
    /// RTP headers truncated before end of buffer
    HeadersTruncated {
        /// The amount of data which was expected to be present (which may vary depending on flags
        /// in the RTP header)
        header_len: usize,
        /// The actual amount of data that was available, which was found to be smaller than
        /// `header_len`
        buffer_len: usize,
    },
    /// The padding header at the end of the packet, if present, specifies the number of padding
    /// bytes, including itself, and therefore cannot be less than `1`, or greater than the
    /// available space.
    PaddingLengthInvalid(u8),
}

impl<'a> RtpReader<'a> {
    /// An RTP packet header is no fewer than 12 bytes long
    pub const MIN_HEADER_LEN: usize = 12;
    const EXTENSION_HEADER_LEN: usize = 4;

    /// Tries to construct a new `RtpHeader` instance, or an `RtpReaderError` if the RTP data is
    /// malformed.
    ///
    /// In particular, if there is too little data in the given buffer, such that some later
    /// attempt to access an RTP header field would need to access bytes that are not available,
    /// then this method will fail up front, rather than allowing attempts to access any header
    /// field to fail later on.
    pub fn new(b: &'a [u8]) -> Result<RtpReader<'_>, RtpReaderError> {
        if b.len() < Self::MIN_HEADER_LEN {
            return Err(RtpReaderError::BufferTooShort(b.len()));
        }
        let r = RtpReader { buf: b };
        if r.version() != 2 {
            return Err(RtpReaderError::UnsupportedVersion(r.version()));
        }
        if r.extension_flag() {
            let extension_start = r.csrc_end() + Self::EXTENSION_HEADER_LEN;
            if extension_start > b.len() {
                return Err(RtpReaderError::HeadersTruncated {
                    header_len: extension_start,
                    buffer_len: b.len(),
                });
            }
            let extension_end = extension_start + r.extension_len();
            if extension_end > b.len() {
                return Err(RtpReaderError::HeadersTruncated {
                    header_len: extension_end,
                    buffer_len: b.len(),
                });
            }
        }
        if r.payload_offset() > b.len() {
            return Err(RtpReaderError::HeadersTruncated {
                header_len: r.payload_offset(),
                buffer_len: b.len(),
            });
        }
        if r.padding_flag() {
            let post_header_bytes = b.len() - r.payload_offset();
            // with 'padding' flag set, there must be at least a single byte after the headers to
            // hold the padding length
            if post_header_bytes == 0 {
                return Err(RtpReaderError::HeadersTruncated {
                    header_len: r.payload_offset(),
                    buffer_len: b.len() - 1,
                });
            }
            let pad_len = r.padding_len()?;

            if r.payload_offset() + pad_len as usize > b.len() {
                return Err(RtpReaderError::PaddingLengthInvalid(pad_len));
            }
        }
        Ok(r)
    }

    /// Version field value (currently only version 2 is supported, so other values will not be
    /// seen from this release of `rtp-rs`.
    pub fn version(&self) -> u8 {
        (self.buf[0] & 0b1100_0000) >> 6
    }

    /// Flag indicating if padding is present at the end of the payload data.
    fn padding_flag(&self) -> bool {
        (self.buf[0] & 0b0010_0000) != 0
    }
    /// Returns the size of the padding at the end of this packet, or `None` if the padding flag is
    /// not set in the packet header
    pub fn padding(&self) -> Option<u8> {
        if self.padding_flag() {
            Some(self.padding_len().unwrap())
        } else {
            None
        }
    }

    fn extension_flag(&self) -> bool {
        (self.buf[0] & 0b0001_0000) != 0
    }
    /// A count of the number of CSRC fields present in the RTP headers - may be `0`.
    ///
    /// See [csrc()](#method.csrc).
    pub fn csrc_count(&self) -> u8 {
        self.buf[0] & 0b0000_1111
    }
    /// A 'marker', which may have some definition in the specific RTP profile in use
    pub fn mark(&self) -> bool {
        (self.buf[1] & 0b1000_0000) != 0
    }
    /// Indicates the type of content carried in this RTP packet.
    ///
    /// A few types-values are defined in the standard, but in many applications of RTP the value
    /// of this field needs to be agreed between sender and receiver by some mechanism outside of
    /// RTP itself.
    pub fn payload_type(&self) -> u8 {
        self.buf[1] & 0b0111_1111
    }
    /// The sequence number of this particular packet.
    ///
    /// Sequence numbers are 16 bits, and will wrap back to `0` after reaching the maximum 16-bit
    /// value of `65535`.
    ///
    /// Receivers can identify packet losses or reordering by inspecting the value of this field
    /// across a sequence of received packets.  The [`Seq`](struct.Seq.html) wrapper type helps
    /// calling code reason about sequence number problems in the face of any wraparound that might
    /// have legitimately happened.
    pub fn sequence_number(&self) -> u16 {
        (self.buf[2] as u16) << 8 | self.buf[3] as u16
    }
    /// The timestamp of this packet, given in a timebase that relates to the particular
    /// `payload_type` in use.
    ///
    /// It is perfectly possible for successive packets in a sequence to have the same value, or
    /// to have values that differ by arbitrarily large amounts.
    ///
    /// Timestamps are 32 bits, and will wrap back to `0` after reaching the maximum 32 bit value
    /// of `4294967295`.
    pub fn timestamp(&self) -> u32 {
        (self.buf[4] as u32) << 24
            | (self.buf[5] as u32) << 16
            | (self.buf[6] as u32) << 8
            | (self.buf[7] as u32)
    }
    /// The _synchronisation source_ for this packet.  Many applications of RTP do not use this
    /// field.
    pub fn ssrc(&self) -> u32 {
        (self.buf[8] as u32) << 24
            | (self.buf[9] as u32) << 16
            | (self.buf[10] as u32) << 8
            | (self.buf[11] as u32)
    }
    /// A potentially empty list of _contributing sources_ for this packet.  Many applications of
    /// RTP do not use this field.
    pub fn csrc(&self) -> impl Iterator<Item = u32> + '_ {
        self.buf[Self::MIN_HEADER_LEN..]
            .chunks(4)
            .take(self.csrc_count() as usize)
            .map(|b| (b[0] as u32) << 24 | (b[1] as u32) << 16 | (b[2] as u32) << 8 | (b[3] as u32))
    }

    /// Returns the offset of the payload for the packet
    pub fn payload_offset(&self) -> usize {
        let offset = self.csrc_end();
        if self.extension_flag() {
            offset + Self::EXTENSION_HEADER_LEN + self.extension_len()
        } else {
            offset
        }
    }

    fn csrc_end(&self) -> usize {
        Self::MIN_HEADER_LEN + (4 * self.csrc_count()) as usize
    }

    /// Returns the payload data of this RTP packet, excluding the packet's headers and any
    /// optional trailing padding.
    pub fn payload(&self) -> &'a [u8] {
        let pad = if self.padding_flag() {
            // in Self::new(), we already checked this was Ok, and will not attempt an invalid
            // slice below,
            self.padding_len().unwrap() as usize
        } else {
            0
        };
        &self.buf[self.payload_offset()..self.buf.len() - pad]
    }

    fn extension_len(&self) -> usize {
        let offset = self.csrc_end();
        // The 16 bit extension length header gives a length in 32 bit (4 byte) units; 0 is a
        // valid length.
        4 * ((self.buf[offset + 2] as usize) << 8 | (self.buf[offset + 3] as usize))
    }

    // must only be used if padding() returns true
    fn padding_len(&self) -> Result<u8, RtpReaderError> {
        match self.buf[self.buf.len() - 1] {
            0 => Err(RtpReaderError::PaddingLengthInvalid(0)),
            l => Ok(l),
        }
    }

    /// Returns details of the optional RTP header extension field.  If there is an extension,
    /// the first component of the resulting tuple is the extension id, and the second is a
    /// byte-slice for the extension data value, to be interpreted by the application.
    pub fn extension(&self) -> Option<(u16, &'a [u8])> {
        if self.extension_flag() {
            let offset = self.csrc_end();
            let id = (self.buf[offset] as u16) << 8 | (self.buf[offset + 1] as u16);
            let start = offset + 4;
            Some((id, &self.buf[start..start + self.extension_len()]))
        } else {
            None
        }
    }
}
