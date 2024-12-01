//! Factory provisioning record reader (A-03).
//!
//! Author: Sandesh Ghimire <sandesh@soccentric.com>
//! Date:   2026-11-04
//! SRS:    factory identity contract PRV1, A-02/A-03/A-05
//!
//! The factory writes a PRV1 record into bank 2 sector 13 (0x0810_4000, or
//! 0x0800_4000 while bank 2 is swapped in).  The reader checks the header,
//! both CRCs and the UID binding, then formats the identity as the HelloReply
//! detail so the factory boot check can compare it over HWLINK.

use core::fmt::Write;

use crate::platform::board;
use crate::platform::plat;
use crate::protocol::hwlink::frame::crc32;

/// "PRV1", little-endian.
const MAGIC: u32 = 0x3156_5250;
/// Supported header format.
const FORMAT: u16 = 1;
/// Header size in bytes.
const HEADER_SIZE: usize = 32;
/// Largest TLV block in bytes.
const MAX_TLV: usize = 4096;
/// Record address with the banks unswapped.
const ADDR_A: u32 = 0x0810_4000;
/// Record address with bank 2 swapped in.
const ADDR_SWAPPED: u32 = 0x0800_4000;

/// TLV tag: serial number.
const T_SERIAL: u8 = 0x01;
/// TLV tag: product name.
const T_PRODUCT: u8 = 0x02;
/// TLV tag: hardware revision.
const T_HW_REV: u8 = 0x03;
/// TLV tag: provisioning profile.
const T_PROFILE: u8 = 0x06;
/// TLV tag: 6-byte MAC address.
const T_MAC: u8 = 0x10;

/// Decoded PRV1 fields, borrowed from flash.
#[derive(Default)]
struct Identity<'a> {
    /// Record revision from the header.
    revision: u16,
    /// Serial number; required.
    serial: &'a [u8],
    /// Product name; required.
    product: &'a [u8],
    /// Hardware revision.
    hw_revision: &'a [u8],
    /// Provisioning profile.
    profile: &'a [u8],
    /// MAC address, if present with the right size.
    mac: Option<[u8; 6]>,
}

/// `fmt::Write` sink over a byte slice; fails instead of truncating.
struct Buffer<'a> {
    /// Destination bytes.
    data: &'a mut [u8],
    /// Bytes written.
    len: usize,
}

impl Write for Buffer<'_> {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let bytes = text.as_bytes();
        if self.len + bytes.len() >= self.data.len() {
            return Err(core::fmt::Error);
        }
        self.data[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

/// Read one flash word.
fn word(address: u32) -> u32 {
    // SAFETY: fixed flash addresses inside the F767 address map.
    unsafe { core::ptr::read_volatile(address as *const u32) }
}

/// Borrow `len` bytes of flash at `address`.
fn bytes(address: u32, len: usize) -> &'static [u8] {
    // SAFETY: the caller bounds len to the 16 KB identity sector.
    unsafe { core::slice::from_raw_parts(address as *const u8, len) }
}

/// Invalidate the D-cache over the header and the largest TLV block.
fn invalidate(address: u32) {
    // The identity sector holds no live cached data, so invalidating is safe.
    plat::dcache_invalidate(address, (HEADER_SIZE + MAX_TLV) as u32);
}

/// The 96-bit device UID as 12 little-endian bytes.
fn uid_words() -> [u8; 12] {
    let mut uid = [0u8; 12];
    for (index, slot) in uid.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        // SAFETY: the UID is three read-only words at a fixed system address.
        let value = unsafe { core::ptr::read_volatile(board::UID_BASE.add(index)) };
        slot.copy_from_slice(&value.to_le_bytes());
    }
    uid
}

/// Validate and decode the PRV1 record at `address`.
///
/// Returns `None` for a bad magic, format, length, CRC or UID binding, a
/// truncated TLV, or a missing serial or product.
fn parse(address: u32) -> Option<Identity<'static>> {
    invalidate(address);
    if word(address) != MAGIC {
        return None;
    }
    let header = bytes(address, HEADER_SIZE);
    let format = u16::from_le_bytes([header[4], header[5]]);
    let revision = u16::from_le_bytes([header[6], header[7]]);
    let length = u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as usize;
    let tlv_crc = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
    let header_crc = u32::from_le_bytes([header[28], header[29], header[30], header[31]]);
    if format != FORMAT || !(2..=MAX_TLV).contains(&length) || crc32(&header[..28]) != header_crc {
        return None;
    }
    if header[16..28] != uid_words() {
        return None;
    }
    let tlv = bytes(address + HEADER_SIZE as u32, length);
    if crc32(tlv) != tlv_crc {
        return None;
    }

    let mut identity = Identity {
        revision,
        ..Identity::default()
    };
    let mut offset = 0usize;
    while offset + 2 <= tlv.len() {
        let kind = tlv[offset];
        let size = tlv[offset + 1] as usize;
        offset += 2;
        if kind == 0 {
            break;
        }
        if offset + size > tlv.len() {
            return None;
        }
        let value = &tlv[offset..offset + size];
        match kind {
            T_SERIAL => identity.serial = value,
            T_PRODUCT => identity.product = value,
            T_HW_REV => identity.hw_revision = value,
            T_PROFILE => identity.profile = value,
            T_MAC if size == 6 => {
                let mut mac = [0u8; 6];
                mac.copy_from_slice(value);
                identity.mac = Some(mac);
            }
            _ => {}
        }
        offset += size;
    }
    if identity.serial.is_empty() || identity.product.is_empty() {
        return None;
    }
    Some(identity)
}

/// `value` as `&str`; `""` if not UTF-8.
fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or("")
}

/// Format the HelloReply detail into `buf`.
///
/// Returns `"no identity"` without a valid record; output that does not fit
/// is cut short.
pub fn detail(buf: &mut [u8]) -> &str {
    let identity = parse(ADDR_A).or_else(|| parse(ADDR_SWAPPED));
    let mut out = Buffer { data: buf, len: 0 };
    match identity {
        Some(identity) => {
            let _ = write!(out, "PRV1 serial={};mac=", text(identity.serial));
            if let Some(mac) = identity.mac {
                let _ = write!(
                    out,
                    "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                );
            }
            let _ = write!(
                out,
                ";hw={};product={};profile={};revision={}",
                text(identity.hw_revision),
                text(identity.product),
                text(identity.profile),
                identity.revision
            );
        }
        None => {
            let _ = write!(out, "no identity");
        }
    }
    core::str::from_utf8(&out.data[..out.len]).unwrap_or("no identity")
}
