/// Available log levels, in increasing priority.
#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum Level {
    Trace,
    Information,
    Warning,
    Error,
    Fatal,
}

/// Contains the state of a logger.
pub struct Logger {
    level: Level,
}

impl Logger {
    /// Get the current logging level of the logger.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Create a new logger with default values.
    pub fn new(level: Level) -> Logger {
        Logger { level }
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
///
/// let logger = Logger::new(Level::Trace);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_trace!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_trace {
    ($logger:expr, $($arg:tt)*) => {
        let lvl = $crate::Level::Trace;
        let msg = format!($($arg)*);
        let msg = format!("[{}] - <{}> ({}({})) {}", lvl as u8, ::chrono::offset::Local::now().format("%Y-%m-%d|%H:%M:%S"), file!(), line!(), msg);
        $crate::message_base($logger, lvl, &msg);
    };
}

/// Log an information message in both debug and release builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
///
/// let logger = Logger::new(Level::Information);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_info!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_info {
    ($logger:expr, $($arg:tt)*) => {
        let lvl = $crate::Level::Information;
        let msg = format!($($arg)*);
        let msg = format!("[{}] - <{}> ({}({})) {}", lvl as u8, ::chrono::offset::Local::now().format("%Y-%m-%d|%H:%M:%S"), file!(), line!(), msg);
        $crate::message_base($logger, lvl, &msg);
    };
}

/// Log a warning message in both debug and release builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
///
/// let logger = Logger::new(Level::Warning);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_warn!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_warn {
    ($logger:expr, $($arg:tt)*) => {
        let lvl = $crate::Level::Warning;
        let msg = format!($($arg)*);
        let msg = format!("[{}] - <{}> ({}({})) {}", lvl as u8, ::chrono::offset::Local::now().format("%Y-%m-%d|%H:%M:%S"), file!(), line!(), msg);
        $crate::message_base($logger, lvl, &msg);
    };
}

/// Log an error message in both debug and release builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
///
/// let logger = Logger::new(Level::Error);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_err!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_err {
    ($logger:expr, $($arg:tt)*) => {
        let lvl = $crate::Level::Error;
        let msg = format!($($arg)*);
        let msg = format!("[{}] - <{}> ({}({})) {}", lvl as u8, ::chrono::offset::Local::now().format("%Y-%m-%d|%H:%M:%S"), file!(), line!(), msg);
        $crate::message_base($logger, lvl, &msg);
    };
}

/// Log a fatal message in both debug and release builds.
/// This panics the program.
/// # Example
/// ```should_panic
/// use rwlog::Logger;
/// use rwlog::Level;
///
/// let logger = Logger::new(Level::Fatal);
/// let a = 5;
/// let b = 4;
/// rwlog::rel_fatal!(&logger, "Variable a is {a} and b is {}.", b);
/// ```
#[macro_export]
macro_rules! rel_fatal {
    ($logger:expr, $($arg:tt)*) => {
        let lvl = $crate::Level::Fatal;
        let msg = format!($($arg)*);
        let msg = format!("[{}] - <{}> ({}({})) {}", lvl as u8, ::chrono::offset::Local::now().format("%Y-%m-%d|%H:%M:%S"), file!(), line!(), msg);
        $crate::message_base($logger, lvl, &msg);
        std::process::exit(1);
    };
}

/// Log a trace message only in debug builds.
/// # Example
/// ```
/// use rwlog::Logger;
/// use rwlog::Level;
///
/// let logger = Logger::new(Level::Trace);
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
///
/// let logger = Logger::new(Level::Information);
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
///
/// let logger = Logger::new(Level::Warning);
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
///
/// let logger = Logger::new(Level::Error);
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
///
/// let logger = Logger::new(Level::Fatal);
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

/// Base logging function.
/// Requires a logger, the logging level and an already formatted string slice.
pub fn message_base(logger: &Logger, lvl: Level, msg: &str) {
    if lvl >= logger.level() {
        match lvl {
            Level::Trace => println!("\x1B[0m{msg}\x1B[0m"),
            Level::Information => println!("\x1B[32m{msg}\x1B[0m"),
            Level::Warning => println!("\x1B[33m{msg}\x1B[0m"),
            Level::Error => println!("\x1B[31m{msg}\x1B[0m"),
            Level::Fatal => println!("\x1B[35m{msg}\x1B[0m"),
        }
    }
}

// pub use crate::warn;
// pub use err;
// pub use fatal;
// pub use info;
// pub use rel_err;
// pub use rel_fatal;
// pub use rel_info;
// pub use rel_trace;
// pub use rel_warn;
// pub use trace;
