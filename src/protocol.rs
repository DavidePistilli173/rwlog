//! This module describes the protocol used for network logging.

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use chrono::{DateTime, Datelike, Local, TimeZone, Timelike};
use num_enum::TryFromPrimitive;
use std::io::Error;
use std::mem::size_of;
use std::net::{SocketAddr, UdpSocket};

/// Available log levels, in increasing priority.
#[derive(PartialEq, PartialOrd, Clone, Copy, TryFromPrimitive, Debug)]
#[repr(u8)]
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
#[derive(Debug)]
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
#[derive(Debug)]
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
    /// Sender of the message, only used when receiving the message from the network.
    pub sender: Option<String>,
}

/// Data contained in the header of protocol version 1.
struct HeaderV1 {
    /// Version number of the communication protocol.
    pub protocol_version: u8,
    /// Type of message contained in the payload.
    pub message_type: u8,
    /// Number of payloads contained after the header.
    pub message_count: u16,
    /// Year in which the message was generated.
    pub timestamp_year: u16,
    /// Month in which the message was generated.
    pub timestamp_month: u8,
    /// Day in which the message was generated.
    pub timestamp_day: u8,
    /// Hour in which the message was generated.
    pub timestamp_hour: u8,
    /// Minute in which the message was generated.
    pub timestamp_minute: u8,
    /// Second in which the message was generated.
    pub timestamp_second: u8,
    /// Log level of the message.
    pub level: u8,
    /// Nanosecond in which the message was generated.
    pub timestamp_nanosecond: u32,
    /// Line number where the message was generated.
    pub line: u32,
    /// Length of the name of the file where the message was generated.
    pub file_name_len: u16,
    /// Name of the file where the message was generated.
    pub file: String,
}

/// Size of the protocol header (version 1) without the file name.
pub const HEADER_LEN_V1: usize = 22;

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
            buffer.resize(
                HEADER_LEN_V1 + message.file.len() + size_of::<u16>() + message.text.len(),
                0,
            );
            encode_header_v1(
                &message,
                1,
                &mut buffer[..HEADER_LEN_V1 + message.file.len()],
            )?;
            let mut payload = &mut buffer[HEADER_LEN_V1 + message.file.len()..];
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
fn encode_header_v1(
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
    buffer.write_u8(message.level as u8)?;
    buffer.write_u32::<NetworkEndian>(message.timestamp.nanosecond())?;
    buffer.write_u32::<NetworkEndian>(message.line)?;
    buffer.write_u16::<NetworkEndian>(message.file.len() as u16)?;
    buffer.copy_from_slice(message.file.as_bytes());

    Ok(())
}

/// Receive a log packet using protocol version 1 and return it.
/// Protocol version 1 is explained in [send_message_v1].
pub fn receive_message_v1(socket: &UdpSocket) -> Option<Message> {
    let mut buffer = [0; u16::MAX as usize];
    let (received_size, sender) = socket.recv_from(&mut buffer).ok()?;

    let decoded_header = decode_header_v1(&buffer[0..received_size])?;
    if decoded_header.protocol_version != 1
        || decoded_header.message_count == 0
        || Level::try_from(decoded_header.level).is_ok()
    {
        return None;
    }

    match decoded_header.message_type {
        // T2 message
        2 => todo!(),
        // T1 message
        1 => Some(Message {
            timestamp: chrono::Local
                .with_ymd_and_hms(
                    decoded_header.timestamp_year as i32,
                    decoded_header.timestamp_month as u32,
                    decoded_header.timestamp_day as u32,
                    decoded_header.timestamp_hour as u32,
                    decoded_header.timestamp_minute as u32,
                    decoded_header.timestamp_second as u32,
                )
                .earliest()?
                .with_nanosecond(decoded_header.timestamp_nanosecond)?,
            level: decoded_header.level.try_into().ok()?,
            text: "".to_string(),
            value: None,
            file: decoded_header.file,
            line: decoded_header.line,
            sender: Some(sender.to_string()),
        }),
        _ => None,
    }
}

fn decode_header_v1(mut buffer: &[u8]) -> Option<HeaderV1> {
    if buffer.len() < HEADER_LEN_V1 {
        return None;
    }

    let mut decoded_header = HeaderV1 {
        protocol_version: buffer.read_u8().ok()?,
        message_type: buffer.read_u8().ok()?,
        message_count: buffer.read_u16::<NetworkEndian>().ok()?,
        timestamp_year: buffer.read_u16::<NetworkEndian>().ok()?,
        timestamp_month: buffer.read_u8().ok()?,
        timestamp_day: buffer.read_u8().ok()?,
        timestamp_hour: buffer.read_u8().ok()?,
        timestamp_minute: buffer.read_u8().ok()?,
        timestamp_second: buffer.read_u8().ok()?,
        level: buffer.read_u8().ok()?,
        timestamp_nanosecond: buffer.read_u32::<NetworkEndian>().ok()?,
        line: buffer.read_u32::<NetworkEndian>().ok()?,
        file_name_len: buffer.read_u16::<NetworkEndian>().ok()?,
        file: "".to_string(),
    };

    decoded_header.file =
        String::from_utf8(buffer[..(decoded_header.file_name_len as usize)].to_vec()).ok()?;

    Some(decoded_header)
}
