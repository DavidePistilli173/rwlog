//! Simple crate for logging messages both in debug and/or release builds.
//!
//! Currently, 3 log targets are supported:
//! - Console => Coloured formatted output on the console.
//! - File => Formatted text file.
//! - Network => Send the log message on the network so that it can be processed somewhere else.
//!
//! For network logging this crate uses a custom protocol.

mod protocol;

pub use crate::protocol::Level;
pub use crate::protocol::Message;

use crossbeam::channel::{bounded, Sender};
use std::fs::File;
use std::io::{LineWriter, Write};
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;
use std::thread;

/// Maximum number of log messages that can be queued.
pub const MESSAGE_BUFFER_SIZE: usize = 1024;

/// Available log destinations.
/// Note that file logging is notably slower than the other options.
#[derive(Clone, Copy)]
pub enum Target {
    /// Log to the console.
    Console,
    /// Log to a file (by default "log.txt" in the executable's folder).
    File,
    /// Log to the network using a custom packet format.
    Network,
}

/// Settings for a network logger.
pub struct NetworkSettings {
    pub local_socket: String,
    pub destination_socket: String,
}

/// Contains the state of a logger.
#[derive(Clone)]
pub struct Logger {
    level: Level,
    target: Target,
    pub channel: Sender<Message>,
}

impl Logger {
    /// Get the current logging level.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Get the current log target.
    pub fn target(&self) -> Target {
        self.target
    }

    /// Create a new logger that targets the console.
    pub fn to_console(level: Level) -> Logger {
        let (sender, receiver) = bounded::<Message>(MESSAGE_BUFFER_SIZE);

        // Spawn the thread that will process all messages.
        let _thread = thread::spawn(move || loop {
            // Receive the next log message.
            let message = match receiver.recv() {
                Ok(x) => x,
                Err(_) => return,
            };

            // Print the received message to console.
            if message.level >= level {
                match message.level {
                    Level::Trace => {
                        println!(
                            "\x1B[0m[{}] - <{}> ({}({})) {}\x1B[0m",
                            message.level as u8,
                            message.timestamp.format("%Y-%m-%d|%H:%M:%S.%f"),
                            message.file,
                            message.line,
                            message.text
                        )
                    }
                    Level::Information => {
                        println!(
                            "\x1B[32m[{}] - <{}> ({}({})) {}\x1B[0m",
                            message.level as u8,
                            message.timestamp.format("%Y-%m-%d|%H:%M:%S.%f"),
                            message.file,
                            message.line,
                            message.text
                        )
                    }
                    Level::Warning => {
                        println!(
                            "\x1B[33m[{}] - <{}> ({}({})) {}\x1B[0m",
                            message.level as u8,
                            message.timestamp.format("%Y-%m-%d|%H:%M:%S.%f"),
                            message.file,
                            message.line,
                            message.text
                        )
                    }
                    Level::Error => {
                        println!(
                            "\x1B[31m[{}] - <{}> ({}({})) {}\x1B[0m",
                            message.level as u8,
                            message.timestamp.format("%Y-%m-%d|%H:%M:%S.%f"),
                            message.file,
                            message.line,
                            message.text
                        )
                    }
                    Level::Fatal => {
                        println!(
                            "\x1B[35m[{}] - <{}> ({}({})) {}\x1B[0m",
                            message.level as u8,
                            message.timestamp.format("%Y-%m-%d|%H:%M:%S.%f"),
                            message.file,
                            message.line,
                            message.text
                        )
                    }
                }
            }
        });

        Logger {
            level,
            target: Target::Console,
            channel: sender,
        }
    }

    /// Create a new logger that targets a file.
    pub fn to_file(level: Level, path: &str) -> Logger {
        let (sender, receiver) = bounded::<Message>(MESSAGE_BUFFER_SIZE);

        // Initialise the log file (at the moment, the actual file path is fixed).
        let log_file = File::create(path).expect("Failed to create log file.");
        let mut log_writer = LineWriter::new(log_file);

        let _thread = thread::spawn(move || loop {
            // Receive the next log message.
            let message = match receiver.recv() {
                Ok(x) => x,
                Err(_) => return,
            };

            // Write the received message to file.
            if message.level >= level {
                match writeln!(
                    log_writer,
                    "[{}] - <{}> ({}({})) {}",
                    message.level as u8,
                    message.timestamp.format("%Y-%m-%d|%H:%M:%S.%f"),
                    message.file,
                    message.line,
                    message.text
                ) {
                    Ok(_) => (),
                    Err(_) => return,
                };
            }
        });

        Logger {
            level,
            target: Target::File,
            channel: sender,
        }
    }

    /// Create a new logger that targets a network socket.
    pub fn to_network(level: Level, settings: &NetworkSettings) -> Logger {
        let (sender, receiver) = bounded::<Message>(MESSAGE_BUFFER_SIZE);

        // Initialise the socket.
        let socket =
            UdpSocket::bind(&settings.local_socket).expect("Failed to bind local logging socket.");
        let destination = SocketAddr::from_str(&settings.destination_socket)
            .expect("Failed to parse log destination address.");

        let _thread = thread::spawn(move || loop {
            // Receive the next log message.
            let message = match receiver.recv() {
                Ok(x) => x,
                Err(_) => return,
            };

            // Write the received message to file.
            if message.level >= level {
                protocol::send_message_v1(&socket, &destination, &message)
                    .expect("Failed to send log message to the network.");
            }
        });

        Logger {
            level,
            target: Target::Network,
            channel: sender,
        }
    }
}

/// Log a trace message in both debug and release builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Trace, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_trace!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_trace {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);

        let msg = $crate::Message {
            timestamp: ::chrono::offset::Local::now(),
            level: $crate::Level::Trace,
            text: msg,
            value: None,
            file: file!().to_string(),
            line: line!()
        };

        $logger.channel.send(msg).expect("Logger thread unreachable.");
    };
}

/// Log an information message in both debug and release builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Information, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_info!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_info {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);

        let msg = $crate::Message {
            timestamp: ::chrono::offset::Local::now(),
            level: $crate::Level::Information,
            text: msg,
            value: None,
            file: file!().to_string(),
            line: line!()
        };

        $logger.channel.send(msg).expect("Logger thread unreachable.");
    };
}

/// Log a warning message in both debug and release builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Warning, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_warn!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_warn {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);

        let msg = $crate::Message {
            timestamp: ::chrono::offset::Local::now(),
            level: $crate::Level::Warning,
            text: msg,
            value: None,
            file: file!().to_string(),
            line: line!()
        };

        $logger.channel.send(msg).expect("Logger thread unreachable.");
    };
}

/// Log an error message in both debug and release builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Error, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_err!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_err {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);

        let msg = $crate::Message {
            timestamp: ::chrono::offset::Local::now(),
            level: $crate::Level::Error,
            text: msg,
            value: None,
            file: file!().to_string(),
            line: line!()
        };

        $logger.channel.send(msg).expect("Logger thread unreachable.");
    };
}

/// Log a fatal message in both debug and release builds.
/// This panics the program.
/// # Example
/// ```should_panic
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Fatal, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_fatal!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_fatal {
    ($logger:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);

        let msg = $crate::Message {
            timestamp: ::chrono::offset::Local::now(),
            level: $crate::Level::Fatal,
            text: msg,
            value: None,
            file: file!().to_string(),
            line: line!()
        };

        $logger.channel.send(msg).expect("Logger thread unreachable.");
        std::thread::sleep(std::time::Duration::from_millis(1000));
        std::process::exit(1);
    };
}

/// Log a trace message only in debug builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Trace, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::trace!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! trace {
    ($logger:expr, $($arg:tt)*) => { $crate::rel_trace!($logger, $($arg)*); };
}

/// Log an information message only in debug builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Information, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::info!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)*) => { $crate::rel_info!($logger, $($arg)*); };
}

/// Log a warning message only in debug builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Warning, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::warn!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! warn {
    ($logger:expr, $($arg:tt)*) => { $crate::rel_warn!($logger, $($arg)*); };
}

/// Log an error message only in debug builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Error, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::err!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! err {
    ($logger:expr, $($arg:tt)*) => { $crate::rel_err!($logger, $($arg)*); };
}

/// Log a fatal message only in debug builds.
/// This panics the program.
/// # Example
/// ```should_panic
/// use rwlog::Logger;
/// use rwlog::Level;
/// use rwlog::Target;
///
/// let logger = Logger::new(Level::Fatal, Target::Console);
/// let a = 5;
/// let b = 4;
/// rwlog::fatal!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! fatal {
    ($logger:expr, $($arg:tt)*) => { $crate::rel_fatal!($logger, $($arg)*); };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {};
}
