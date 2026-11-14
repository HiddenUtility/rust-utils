//! Rust Text Logger crate.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LoggerError {
    Io(std::io::Error),
    Other(String),
}

impl fmt::Display for LoggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoggerError::Io(e) => write!(f, "IO error: {}", e),
            LoggerError::Other(s) => write!(f, "Other error: {}", s),
        }
    }
}

impl std::error::Error for LoggerError {}

impl From<std::io::Error> for LoggerError {
    fn from(err: std::io::Error) -> Self {
        LoggerError::Io(err)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LogLevel {
    CRITICAL,
    ERROR,
    WARNING,
    INFO,
    DEBUG,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::CRITICAL => write!(f, "CRITICAL"),
            LogLevel::ERROR => write!(f, "ERROR"),
            LogLevel::WARNING => write!(f, "WARNING"),
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::DEBUG => write!(f, "DEBUG"),
        }
    }
}

pub struct LogRecode {
    pub timestamp: String,
    pub level: LogLevel,
    pub msg: String,
    pub pid: i32,
}

impl LogRecode {
    pub fn critical(msg: &str) -> Self {
        unimplemented!()
    }
    pub fn error(msg: &str) -> Self {
        unimplemented!()
    }
    pub fn warning(msg: &str) -> Self {
        unimplemented!()
    }
    pub fn info(msg: &str) -> Self {
        unimplemented!()
    }
    pub fn debug(msg: &str) -> Self {
        unimplemented!()
    }
    pub fn to_log(&self) -> String {
        unimplemented!()
    }
}

pub struct Logger {
    log_path: PathBuf,
    recodes: Vec<LogRecode>,
    max_file_num: u32,
    max_file_size: u32,
    realtime: bool,
    now_size: u32,
}

impl Logger {
    pub fn new<P: AsRef<std::path::Path>>(log_path: P) -> std::io::Result<Self> {
        unimplemented!()
    }
    pub fn new_with<P: AsRef<std::path::Path>>(
        log_path: P,
        max_file_num: u32,
        max_file_size: u32,
        realtime: bool,
    ) -> std::io::Result<Self> {
        unimplemented!()
    }
    pub fn critical(&mut self, msg: &str) -> std::io::Result<()> {
        unimplemented!()
    }
    pub fn error(&mut self, msg: &str) -> std::io::Result<()> {
        unimplemented!()
    }
    pub fn warning(&mut self, msg: &str) -> std::io::Result<()> {
        unimplemented!()
    }
    pub fn info(&mut self, msg: &str) -> std::io::Result<()> {
        unimplemented!()
    }
    pub fn debug(&mut self, msg: &str) -> std::io::Result<()> {
        unimplemented!()
    }
    pub fn out(&mut self) -> std::io::Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::CRITICAL), "CRITICAL");
        assert_eq!(format!("{}", LogLevel::ERROR), "ERROR");
        assert_eq!(format!("{}", LogLevel::WARNING), "WARNING");
        assert_eq!(format!("{}", LogLevel::INFO), "INFO");
        assert_eq!(format!("{}", LogLevel::DEBUG), "DEBUG");
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn test_log_record_creation() {
        let _record = LogRecode::info("test");
    }
}
