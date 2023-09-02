//! Simple crate for logging messages both in debug and/or release builds.
//!
//! Currently, 3 log targets are supported:
//! - Console => Coloured formatted output on the console.
//! - File => Formatted text file, currently fixed to "log.txt" in the executable's folder.
//! - Network => Send the log message on the network so that it can be processed somewhere else.
//!
//! For network logging this crate uses a custom protocol.
//!

use chrono::{DateTime, Local};
use crossbeam::channel::{bounded, Sender};
use std::fs::File;
use std::io::{LineWriter, Write};
use std::thread;

/// Maximum number of log messages that can be queued.
pub const MESSAGE_BUFFER_SIZE: usize = 1024;

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

/// Available log destinations.
/// Note that file logging is notably slower than the other options.
pub enum Target {
    /// Log to the console.
    Console,
    /// Log to a file (by default "log.txt" in the executable's folder).
    File,
    /// Log to the network using a custom packet format.
    Network,
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

/// Internal structure that holds all information for a single log message.
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

/// Contains the state of a logger.
#[derive(Clone)]
pub struct Logger {
    level: Level,
    pub channel: Sender<Message>,
}

impl Logger {
    /// Get the current logging level of the logger.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Create a new logger with specific values.
    pub fn new(level: Level, target: Target) -> Logger {
        let (sender, receiver) = bounded::<Message>(MESSAGE_BUFFER_SIZE);

        // Spawn the thread that will process all messages.
        let _thread = match target {
            Target::Console => thread::spawn(move || loop {
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
            }),
            Target::File => thread::spawn(move || {
                // Initialise the log file (at the moment, the actual file path is fixed).
                let log_file = File::create("log.txt").expect("Failed to create log file.");
                let mut log_writer = LineWriter::new(log_file);

                // Main logger loop.
                loop {
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
                }
            }),
            Target::Network => todo!(),
        };

        Logger {
            level,
            channel: sender,
        }
    }

    /// Set the logging level of the logger.
    pub fn set_level(&mut self, new_level: Level) {
        self.level = new_level;
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
