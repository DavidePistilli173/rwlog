//! This module describes the protocol used for network logging.

use byteorder::{NativeEndian, NetworkEndian, WriteBytesExt};
use chrono::{DateTime, Datelike, Local, Timelike};
use std::io::Error;
use std::mem::size_of;
use std::net::{SocketAddr, UdpSocket};

/// Available log levels, in increasing priority.
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum Level {
    /// Log all messages with Trace priority or higher.
    Trace,
    /// Log all messages with Information priority or higher.
    Information,
    /// Log all messages with Warning priority or higher.
    Warning,
    /// Log all messages with Error priority or higher.
    Error,
    /// Log all messages with Fatal priority or higher.
    Fatal,
}

/// Supported variable types for a T2 message.
pub enum SupportedTypes {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// Structure that holds all information for a single log message.
pub struct Message {
    /// Raw timestamp value.
    pub timestamp: DateTime<Local>,
    /// Priority of the message.
    pub level: Level,
    /// Formatted text associated with the message.
    pub text: String,
    /// Optional value associated with the message.
    pub value: Option<SupportedTypes>,
    /// File where the message originated.
    pub file: String,
    /// Line where the message originated.
    pub line: u32,
}

/// Size of the protocol header (version 1).
pub const HEADER_LEN_V1: usize = 16;

/// Encode a log packet using protocol version 1 and send it to the specified address.
/// Protocol version 1 is organised as follows:
/// - Header
/// - Payload
///
/// The header is formatted as explained in [encode_header_v1].
///
/// Currently, two types of payloads are supported:
/// - T1 => Pre-formatted text message.
/// - T2 => Value with a name.
///
/// T1 payloads are formatted as follows:
/// - text_length: u16,
/// - text: u8[text_length]
pub fn send_message_v1(
    socket: &UdpSocket,
    destination: &SocketAddr,
    message: &Message,
) -> Result<(), Error> {
    let mut buffer = Vec::<u8>::new();

    match &message.value {
        // T2 message
        Some(_) => todo!(),
        // T1 message
        None => {
            buffer.resize(HEADER_LEN_V1 + size_of::<u16>() + message.text.len(), 0);
            encode_header_v1(&message, 1, &mut buffer[..HEADER_LEN_V1])?;
            let mut payload = &mut buffer[HEADER_LEN_V1..];
            payload.write_u16::<NetworkEndian>(message.text.len() as u16)?;
            payload.copy_from_slice(message.text.as_bytes());
        }
    }

    socket.send_to(&buffer, &destination)?;

    Ok(())
}

/// Encode a header of protocol version 1.
/// A Header v1 is formatted as follows:
/// - protocol_version: u8
/// - message_type: u8
/// - message_count: u16,
/// - year: u16,
/// - month: u8,
/// - day: u8,
/// - hour: u8,
/// - minute: u8,
/// - second: u8,
/// - nanosecond: u32,
/// - log_level: u8,
/// Notes:
/// - Currently, only a message count of 1 is supported.
pub fn encode_header_v1(
    message: &Message,
    message_type: u8,
    mut buffer: &mut [u8],
) -> Result<(), Error> {
    buffer.write_u8(1)?; // protocol_version
    buffer.write_u8(message_type)?;
    buffer.write_u16::<NetworkEndian>(1)?; // message count
    buffer.write_u16::<NetworkEndian>(message.timestamp.year() as u16)?;
    buffer.write_u8(message.timestamp.month() as u8)?;
    buffer.write_u8(message.timestamp.day() as u8)?;
    buffer.write_u8(message.timestamp.hour() as u8)?;
    buffer.write_u8(message.timestamp.minute() as u8)?;
    buffer.write_u8(message.timestamp.second() as u8)?;
    buffer.write_u32::<NetworkEndian>(message.timestamp.nanosecond())?;
    buffer.write_u8(message.level as u8)?;

    Ok(())
}
