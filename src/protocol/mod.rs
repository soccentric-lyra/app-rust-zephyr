//! HWLINK wire protocol on USART3.
//!
//! `hwlink` holds the COBS/CRC-32 framing and a hand-written FlatBuffers codec
//! for `protocol/hwlink.fbs`.

pub mod hwlink;
