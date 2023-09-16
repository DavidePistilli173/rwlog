//! Simple crate for logging messages both in debug and/or release builds.
//!
//! Currently, 3 log targets are supported:
//! - Console => Coloured formatted output on the console.
//! - File => Formatted text file.
//! - Network => Send the log message on the network so that it can be processed somewhere else.
//!
//! For network logging this crate uses a custom protocol.

mod protocol;
pub mod receiver;
pub mod sender;

pub use crate::protocol::Level;
pub use crate::protocol::Message;
