//! Module for receiving log messages from the network.
//! Log messages must use the protocol specified in the [module@protocol] module.

use crate::protocol;
use crate::protocol::Level;
use crate::protocol::Message;
use crossbeam::channel::{bounded, Receiver, RecvTimeoutError};
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

pub const MESSAGE_BUFFER_SIZE: usize = 1024;

/// Contains the state of a logger.
#[derive(Clone)]
pub struct Logger {
    level: Level,
    pub channel: Receiver<Message>,
}

impl Logger {
    /// Get the current logging level.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Create a new logger that will receive log message with high enough priority on the specified socket.
    pub fn new(level: Level, socket: &str) -> Logger {
        let (sender, receiver) = bounded::<Message>(MESSAGE_BUFFER_SIZE);

        // Initialise the socket.
        let socket = UdpSocket::bind(&socket).expect("Failed to bind local logging socket.");

        let _thread = thread::spawn(move || loop {
            // Receive the next log message.
            if let Some(message) = protocol::receive_message_v1(&socket) {
                if message.level >= level {
                    sender
                        .send(message)
                        .expect("Failed to send received log message to the application.");
                }
            }
        });

        Logger {
            level,
            channel: receiver,
        }
    }

    /// Get the next message in the received queue up to an optional timeout.
    pub fn next_message(&self, timeout: Option<Duration>) -> Result<Message, RecvTimeoutError> {
        match timeout {
            Some(x) => self.channel.recv_timeout(x),
            None => self
                .channel
                .recv()
                .map_err(|_err| RecvTimeoutError::Disconnected),
        }
    }
}
