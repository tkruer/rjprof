use std::sync::OnceLock;

// Simple logger for the library that doesn't depend on external crates
pub struct Logger {
    enabled: bool,
}

impl Logger {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn info(&self, message: &str) {
        if self.enabled {
            println!("[INFO] {}", message);
        }
    }

    pub fn warn(&self, message: &str) {
        if self.enabled {
            eprintln!("[WARN] {}", message);
        }
    }

    pub fn error(&self, message: &str) {
        eprintln!("[ERROR] {}", message);
    }

    pub fn debug(&self, message: &str) {
        if self.enabled {
            println!("[DEBUG] {}", message);
        }
    }

    pub fn status(&self, message: &str) {
        if self.enabled {
            println!("Status: {}", message);
        }
    }

    pub fn result(&self, message: &str) {
        if self.enabled {
            println!("Result: {}", message);
        }
    }

    pub fn section(&self, title: &str) {
        if self.enabled {
            let separator = "=".repeat(title.len() + 8);
            println!("{}", separator);
            println!("    {}", title);
            println!("{}", separator);
        }
    }

    pub fn subsection(&self, title: &str) {
        if self.enabled {
            let separator = "-".repeat(title.len() + 4);
            println!("{}", separator);
            println!("  {}", title);
            println!("{}", separator);
        }
    }

    // Raw output for performance data
    pub fn raw(&self, message: &str) {
        if self.enabled {
            println!("{}", message);
        }
    }
}

static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn init_logger(enabled: bool) {
    GLOBAL_LOGGER.set(Logger::new(enabled)).ok();
}

pub fn get_logger() -> &'static Logger {
    GLOBAL_LOGGER.get_or_init(|| Logger::new(true))
}

// Convenience macros
#[macro_export]
macro_rules! lib_info {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().info(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! lib_warn {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().warn(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! lib_error {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().error(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! lib_debug {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().debug(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! lib_status {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().status(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! lib_result {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().result(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! lib_section {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().section(&format!($($arg)*))
    };
}

#[macro_export]
macro_rules! lib_raw {
    ($($arg:tt)*) => {
        $crate::logger::get_logger().raw(&format!($($arg)*))
    };
}