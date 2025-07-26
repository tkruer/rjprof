use std::fmt::{self, Display};
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN "),
            LogLevel::Info => write!(f, "INFO "),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

pub struct Logger {
    level: LogLevel,
    use_colors: bool,
    quiet: bool,
}

impl Logger {
    pub fn new(level: LogLevel, use_colors: bool, quiet: bool) -> Self {
        Self {
            level,
            use_colors,
            quiet,
        }
    }

    pub fn with_level(level: LogLevel) -> Self {
        Self::new(level, atty::is(atty::Stream::Stdout), false)
    }

    pub fn default() -> Self {
        Self::new(LogLevel::Info, atty::is(atty::Stream::Stdout), false)
    }

    pub fn quiet() -> Self {
        Self::new(LogLevel::Error, false, true)
    }

    pub fn verbose() -> Self {
        Self::new(LogLevel::Debug, atty::is(atty::Stream::Stdout), false)
    }

    fn should_log(&self, level: LogLevel) -> bool {
        if self.quiet && level > LogLevel::Error {
            return false;
        }
        level <= self.level
    }

    fn color_for_level(&self, level: LogLevel) -> &'static str {
        if !self.use_colors {
            return "";
        }
        match level {
            LogLevel::Error => "\x1b[31m", // Red
            LogLevel::Warn => "\x1b[33m",  // Yellow
            LogLevel::Info => "\x1b[36m",  // Cyan
            LogLevel::Debug => "\x1b[35m", // Magenta
            LogLevel::Trace => "\x1b[37m", // White
        }
    }

    fn reset_color(&self) -> &'static str {
        if self.use_colors {
            "\x1b[0m"
        } else {
            ""
        }
    }

    fn log_with_level(&self, level: LogLevel, message: &str) {
        if !self.should_log(level) {
            return;
        }

        let color = self.color_for_level(level);
        let reset = self.reset_color();

        match level {
            LogLevel::Error => {
                eprintln!("{}[{}]{} {}", color, level, reset, message);
                let _ = io::stderr().flush();
            }
            _ => {
                println!("{}[{}]{} {}", color, level, reset, message);
                let _ = io::stdout().flush();
            }
        }
    }

    pub fn error(&self, message: &str) {
        self.log_with_level(LogLevel::Error, message);
    }

    pub fn warn(&self, message: &str) {
        self.log_with_level(LogLevel::Warn, message);
    }

    pub fn info(&self, message: &str) {
        self.log_with_level(LogLevel::Info, message);
    }

    pub fn debug(&self, message: &str) {
        self.log_with_level(LogLevel::Debug, message);
    }

    pub fn trace(&self, message: &str) {
        self.log_with_level(LogLevel::Trace, message);
    }

    // Convenience methods for common profiler outputs
    pub fn config(&self, message: &str) {
        self.info(&format!("Configuration: {}", message));
    }

    pub fn status(&self, message: &str) {
        self.info(&format!("Status: {}", message));
    }

    pub fn result(&self, message: &str) {
        self.info(&format!("Result: {}", message));
    }

    pub fn progress(&self, message: &str) {
        self.debug(&format!("Progress: {}", message));
    }

    pub fn section(&self, title: &str) {
        if !self.should_log(LogLevel::Info) {
            return;
        }

        let separator = "=".repeat(title.len() + 8);
        self.info(&separator);
        self.info(&format!("    {}", title));
        self.info(&separator);
    }

    pub fn subsection(&self, title: &str) {
        if !self.should_log(LogLevel::Info) {
            return;
        }

        let separator = "-".repeat(title.len() + 4);
        self.info(&separator);
        self.info(&format!("  {}", title));
        self.info(&separator);
    }

    // Raw output without log level formatting (for data output)
    pub fn raw(&self, message: &str) {
        if !self.quiet {
            println!("{}", message);
            let _ = io::stdout().flush();
        }
    }

    // Success/failure states
    pub fn success(&self, message: &str) {
        let color = if self.use_colors { "\x1b[32m" } else { "" }; // Green
        let reset = self.reset_color();
        if !self.quiet {
            println!("{}SUCCESS{} {}", color, reset, message);
            let _ = io::stdout().flush();
        }
    }

    pub fn failure(&self, message: &str) {
        let color = if self.use_colors { "\x1b[31m" } else { "" }; // Red
        let reset = self.reset_color();
        eprintln!("{}FAILURE{} {}", color, reset, message);
        let _ = io::stderr().flush();
    }
}

// Global logger instance
use std::sync::OnceLock;
static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn init_logger(logger: Logger) {
    GLOBAL_LOGGER.set(logger).ok();
}

pub fn get_logger() -> &'static Logger {
    GLOBAL_LOGGER.get_or_init(|| Logger::default())
}

// Convenience macros for global logging
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().warn(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().debug(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_config {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().config(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_status {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().status(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_result {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().result(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().success(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_failure {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().failure(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_section {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().section(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_raw {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().raw(&format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        let logger = Logger::with_level(LogLevel::Info);
        assert!(logger.should_log(LogLevel::Error));
        assert!(logger.should_log(LogLevel::Warn));
        assert!(logger.should_log(LogLevel::Info));
        assert!(!logger.should_log(LogLevel::Debug));
        assert!(!logger.should_log(LogLevel::Trace));
    }

    #[test]
    fn test_quiet_mode() {
        let logger = Logger::quiet();
        assert!(logger.should_log(LogLevel::Error));
        assert!(!logger.should_log(LogLevel::Warn));
        assert!(!logger.should_log(LogLevel::Info));
        assert!(!logger.should_log(LogLevel::Debug));
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }
}
