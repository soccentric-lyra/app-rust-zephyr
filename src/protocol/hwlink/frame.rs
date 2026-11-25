//! HWLINK COBS framing and CRC-32/ISO-HDLC.
//!
//! Author: Sandesh Ghimire <sandesh@soccentric.com>
//! SRS:    RLNK-003, RLNK-004; LINK A.2
//!
//! Raw frame: `proto`, `flags`, `seq` (LE u16), payload, CRC-32 (LE) over the
//! preceding bytes.  On the wire it is COBS-encoded and ends with a zero byte.

/// Largest raw (decoded) frame in bytes.
pub const MAX_DECODED: usize = 2048;
/// Largest COBS-encoded frame in bytes, terminator included.
pub const MAX_ENCODED: usize = MAX_DECODED + MAX_DECODED / 254 + 2;

/// Protocol major version in byte 0 of every frame.
pub const PROTO_MAJOR: u8 = 1;
/// Frame is a response.
pub const FLAG_RESPONSE: u8 = 0x01;
/// Frame is an unsolicited event.
pub const FLAG_EVENT: u8 = 0x02;

/// Byte-wise CRC-32 table, reflected polynomial 0xEDB88320.
const CRC_TABLE: [u32; 256] = make_crc_table();

/// Build [`CRC_TABLE`] at compile time.
const fn make_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// CRC-32/ISO-HDLC, identical to Python's `zlib.crc32`.
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc = CRC_TABLE[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

/// COBS-encode `input` into `out` and append the zero terminator.
///
/// Returns the encoded length, or `None` if `out` is too small.
fn cobs_encode(input: &[u8], out: &mut [u8]) -> Option<usize> {
    let mut read = 0usize;
    let mut write = 1usize;
    let mut code = 1u8;
    let mut code_idx = 0usize;

    while read < input.len() {
        if input[read] == 0 {
            *out.get_mut(code_idx)? = code;
            code_idx = write;
            write += 1;
            code = 1;
        } else {
            *out.get_mut(write)? = input[read];
            write += 1;
            code = code.wrapping_add(1);
            if code == 0xFF {
                *out.get_mut(code_idx)? = code;
                code_idx = write;
                write += 1;
                code = 1;
            }
        }
        read += 1;
    }

    *out.get_mut(code_idx)? = code;
    *out.get_mut(write)? = 0;
    Some(write + 1)
}

/// COBS-decode `input` (no terminator) into `out`.
///
/// Returns the decoded length, or `None` for an embedded zero or if `out` is
/// too small.
fn cobs_decode(input: &[u8], out: &mut [u8]) -> Option<usize> {
    let mut read = 0usize;
    let mut write = 0usize;

    while read < input.len() {
        let code = *input.get(read)? as usize;
        if code == 0 {
            return None;
        }
        read += 1;
        for _ in 1..code {
            *out.get_mut(write)? = *input.get(read)?;
            write += 1;
            read += 1;
        }
        if code != 0xFF && read < input.len() {
            *out.get_mut(write)? = 0;
            write += 1;
        }
    }
    Some(write)
}

/// Build one COBS-encoded HWLINK frame in `out`.
///
/// `raw` is scratch for the frame before COBS.  Returns the encoded length,
/// or `None` if the frame exceeds [`MAX_DECODED`] or a buffer is too small.
pub fn encode(
    raw: &mut [u8],
    out: &mut [u8],
    flags: u8,
    seq: u16,
    payload: &[u8],
) -> Option<usize> {
    let len = 4 + payload.len() + 4;
    if len > MAX_DECODED || raw.len() < len {
        return None;
    }

    raw[0] = PROTO_MAJOR;
    raw[1] = flags;
    raw[2..4].copy_from_slice(&seq.to_le_bytes());
    raw[4..4 + payload.len()].copy_from_slice(payload);
    let crc = crc32(&raw[..4 + payload.len()]);
    raw[4 + payload.len()..len].copy_from_slice(&crc.to_le_bytes());
    cobs_encode(&raw[..len], out)
}

/// COBS receive state machine.  A zero byte terminates one frame.
pub struct Decoder {
    /// Encoded bytes of the frame being received.
    buf: [u8; MAX_ENCODED],
    /// Bytes in `buf`.
    len: usize,
    /// The current frame overran `buf`; drop it at the next zero.
    overflow: bool,
}

impl Decoder {
    /// Create an empty decoder.
    pub const fn new() -> Self {
        Self {
            buf: [0; MAX_ENCODED],
            len: 0,
            overflow: false,
        }
    }

    /// Feed one received byte.
    ///
    /// Returns `true` when a complete, still encoded frame is ready; call
    /// [`Decoder::take`] next.  Overlong frames are dropped.
    pub fn push(&mut self, byte: u8) -> bool {
        if byte == 0 {
            let done = self.len != 0 && !self.overflow;
            self.overflow = false;
            if !done {
                self.len = 0;
            }
            return done;
        }

        if self.len < self.buf.len() {
            self.buf[self.len] = byte;
            self.len += 1;
        } else {
            self.overflow = true;
            self.len = 0;
        }
        false
    }

    /// Decode the captured frame into `out` and check length, CRC and version.
    ///
    /// Returns the raw frame length, or `None` if the frame is invalid.
    pub fn take(&mut self, out: &mut [u8]) -> Option<usize> {
        if self.len == 0 {
            return None;
        }
        let encoded = &self.buf[..self.len];
        self.len = 0;
        let len = cobs_decode(encoded, out)?;
        if !(8..=MAX_DECODED).contains(&len) {
            return None;
        }
        let got = u32::from_le_bytes(out[len - 4..len].try_into().ok()?);
        if got != crc32(&out[..len - 4]) {
            return None;
        }
        if out[0] != PROTO_MAJOR {
            return None;
        }
        Some(len)
    }
}

/// Header fields and payload of a raw frame.
pub struct Frame<'a> {
    /// `FLAG_*` bits.
    pub flags: u8,
    /// Sequence number.
    pub seq: u16,
    /// Payload between header and CRC.
    pub payload: &'a [u8],
}

/// Split a CRC-checked raw frame; `None` if too short or with unknown flags.
pub fn parse(frame: &[u8]) -> Option<Frame<'_>> {
    if frame.len() < 8 || frame[1] & !(FLAG_RESPONSE | FLAG_EVENT) != 0 {
        return None;
    }
    Some(Frame {
        flags: frame[1],
        seq: u16::from_le_bytes([frame[2], frame[3]]),
        payload: &frame[4..frame.len() - 4],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(payload: &[u8], flags: u8, seq: u16) -> ([u8; MAX_DECODED], [u8; MAX_ENCODED], usize) {
        let mut raw = [0u8; MAX_DECODED];
        let mut encoded = [0u8; MAX_ENCODED];
        let len = encode(&mut raw, &mut encoded, flags, seq, payload)
            .expect("frame must fit the test buffers");
        (raw, encoded, len)
    }

    fn decode(encoded: &[u8]) -> Option<([u8; MAX_DECODED], usize)> {
        let mut decoder = Decoder::new();
        let mut done = false;
        for &byte in encoded {
            done = decoder.push(byte);
        }
        assert!(done, "encoded frame must end with the zero delimiter");
        let mut out = [0u8; MAX_DECODED];
        let len = decoder.take(&mut out)?;
        Some((out, len))
    }

    #[test]
    fn crc_matches_the_standard_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn cobs_examples() {
        let mut out = [0u8; 16];
        let mut n = cobs_encode(&[0x00], &mut out).unwrap();
        assert_eq!(&out[..n], &[0x01, 0x01, 0x00]);

        n = cobs_encode(&[0x01, 0x00, 0x02], &mut out).unwrap();
        assert_eq!(&out[..n], &[0x02, 0x01, 0x02, 0x02, 0x00]);
    }

    #[test]
    fn cobs_rejects_bad_bodies() {
        let mut out = [0u8; 16];
        assert_eq!(cobs_decode(&[0x00], &mut out), None);
        assert_eq!(cobs_decode(&[0x03, 0x01], &mut out), None);
        assert_eq!(cobs_decode(&[0x05, 0x01], &mut out), None);
    }

    #[test]
    fn roundtrip_preserves_header_and_payload() {
        let payload = [0x00u8, 0x01, 0x02, 0xFE, 0xFF, 0x00];
        let (_raw, encoded, len) = build(&payload, FLAG_RESPONSE, 0x1234);
        let (out, got) = decode(&encoded[..len]).unwrap();
        let frame = parse(&out[..got]).unwrap();
        assert_eq!(frame.flags, FLAG_RESPONSE);
        assert_eq!(frame.seq, 0x1234);
        assert_eq!(frame.payload, &payload);
    }

    #[test]
    fn cobs_block_boundaries() {
        for len in 253..=255usize {
            let payload = [0x5Au8; 255];
            let (_raw, encoded, n) = build(&payload[..len], 0, 1);
            let (out, got) = decode(&encoded[..n]).unwrap();
            let frame = parse(&out[..got]).unwrap();
            assert_eq!(frame.payload, &payload[..len]);
        }
    }

    #[test]
    fn rejects_corruption_and_wrong_protocol() {
        let (mut raw, encoded, len) = build(&[0x01, 0x02, 0x03], 0, 7);
        let mut corrupt = encoded;
        corrupt[2] ^= 0xFF;
        assert_eq!(decode(&corrupt[..len]), None);

        raw[0] = PROTO_MAJOR + 1;
        let crc = crc32(&raw[..7]);
        raw[7..11].copy_from_slice(&crc.to_le_bytes());
        let mut wrong = [0u8; MAX_ENCODED];
        let n = cobs_encode(&raw[..11], &mut wrong).unwrap();
        assert_eq!(decode(&wrong[..n]), None);
    }

    #[test]
    fn parse_rejects_reserved_flags_and_short_frames() {
        let (raw, _encoded, _len) = build(&[0x01], 0x04, 1);
        assert!(parse(&raw[..9]).is_none());
        assert!(parse(&[0u8; 7]).is_none());
    }

    #[test]
    fn rejects_oversized_frames() {
        let payload = [0u8; 2041];
        let mut raw = [0u8; MAX_DECODED];
        let mut encoded = [0u8; MAX_ENCODED];
        assert!(encode(&mut raw, &mut encoded, 0, 1, &payload).is_none());
    }
}
