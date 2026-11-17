use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    DEBUG = 1,
    INFO = 2,
    WARNING = 3,
    ERROR = 4,
    CRITICAL = 5,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARNING => "WARNING",
            LogLevel::ERROR => "ERROR",
            LogLevel::CRITICAL => "CRITICAL",
        };
        write!(f, "{}", s)
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
        Self::new(LogLevel::CRITICAL, msg)
    }
    pub fn error(msg: &str) -> Self {
        Self::new(LogLevel::ERROR, msg)
    }
    pub fn warning(msg: &str) -> Self {
        Self::new(LogLevel::WARNING, msg)
    }
    pub fn info(msg: &str) -> Self {
        Self::new(LogLevel::INFO, msg)
    }
    pub fn debug(msg: &str) -> Self {
        Self::new(LogLevel::DEBUG, msg)
    }

    fn new(level: LogLevel, msg: &str) -> Self {
        let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.6f").to_string();
        let pid = process::id() as i32;
        Self {
            timestamp,
            level,
            msg: msg.to_string(),
            pid,
        }
    }

    pub fn to_log(&self) -> String {
        format!("{} [{}] {}", self.timestamp, self.level, self.msg)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum LoggerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Path error: {0}")]
    PathError(String),
}

pub struct Logger {
    path: PathBuf,
    log_level: LogLevel,
    max_file_num: usize,
    max_file_size: u64,
    realtime: bool,
    recodes: Vec<LogRecode>,
    now_size: u64,
}

impl Logger {
    pub fn new(
        path: &Path,
        log_level: LogLevel,
        max_file_num: usize,
        max_file_size: u64,
        realtime: bool,
    ) -> Result<Self, LoggerError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let now_size = if path.exists() {
            fs::metadata(path)?.len()
        } else {
            0
        };

        Ok(Self {
            path: path.to_path_buf(),
            log_level,
            max_file_num,
            max_file_size,
            realtime,
            recodes: Vec::new(),
            now_size,
        })
    }

    pub fn new_with(
        path: &Path,
        log_level: LogLevel,
        max_file_num: usize,
        max_file_size: u64,
        realtime: bool,
        _buffer_size: usize,
    ) -> Result<Self, LoggerError> {
        Self::new(path, log_level, max_file_num, max_file_size, realtime)
    }

    pub fn critical(&mut self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::CRITICAL, msg)
    }
    pub fn error(&mut self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::ERROR, msg)
    }
    pub fn warning(&mut self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::WARNING, msg)
    }
    pub fn info(&mut self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::INFO, msg)
    }
    pub fn debug(&mut self, msg: &str) -> Result<(), LoggerError> {
        self.log(LogLevel::DEBUG, msg)
    }

    fn log(&mut self, level: LogLevel, msg: &str) -> Result<(), LoggerError> {
        if level >= self.log_level {
            let rec = LogRecode::new(level, msg);
            if self.realtime {
                self.recodes.push(rec);
                self.out()?;
                self.recodes.clear();
            } else {
                self.recodes.push(rec);
            }
        }
        Ok(())
    }

    pub fn write_recs(&mut self, recodes: &[LogRecode]) -> Result<(), LoggerError> {
        for rec in recodes {
            if rec.level >= self.log_level {
                self.recodes.push(LogRecode {
                    timestamp: rec.timestamp.clone(),
                    level: rec.level,
                    msg: rec.msg.clone(),
                    pid: rec.pid,
                });
            }
        }
        if self.realtime {
            self.out()?;
            self.recodes.clear();
        }
        Ok(())
    }

    pub fn out(&mut self) -> Result<(), LoggerError> {
        if self.recodes.is_empty() {
            return Ok(());
        }

        let mut new_content = String::new();
        let mut added_size = 0;
        for rec in &self.recodes {
            let line = rec.to_log();
            new_content.push_str(&line);
            new_content.push('\n');
            added_size += line.len() + 1;
        }

        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.path)?;

        file.write_all(new_content.as_bytes())?;
        self.now_size += added_size as u64;
        self.recodes.clear();

        if self.now_size > self.max_file_size {
            self.rotate()?;
        }

        Ok(())
    }

    fn rotate(&mut self) -> Result<(), LoggerError> {
        if self.max_file_num <= 1 {
            fs::File::create(&self.path)?;
            self.now_size = 0;
            return Ok(());
        }

        for i in (1..self.max_file_num).rev() {
            let old_path = self.path.with_extension(format!("log.{}", i));
            if old_path.exists() {
                let new_path = self.path.with_extension(format!("log.{}", i + 1));
                if i + 1 >= self.max_file_num {
                    fs::remove_file(old_path)?;
                } else {
                    fs::rename(old_path, new_path)?;
                }
            }
        }

        let first_rotated_path = self.path.with_extension("log.1");
        fs::rename(&self.path, first_rotated_path)?;

        self.now_size = 0;

        Ok(())
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.out();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::INFO.to_string(), "INFO");
        assert_eq!(LogLevel::CRITICAL.to_string(), "CRITICAL");
    }

    #[test]
    fn test_log_level_comparison() {
        assert!(LogLevel::CRITICAL > LogLevel::ERROR);
        assert!(LogLevel::ERROR > LogLevel::WARNING);
        assert!(LogLevel::WARNING > LogLevel::INFO);
        assert!(LogLevel::INFO > LogLevel::DEBUG);
    }

    #[test]
    fn test_log_recode_creation() {
        let rec = LogRecode::critical("test message");
        assert_eq!(rec.level, LogLevel::CRITICAL);
        assert_eq!(rec.msg, "test message");
        assert!(!rec.timestamp.is_empty());
        assert!(rec.pid > 0);
    }

    #[test]
    fn test_log_recode_to_log() {
        let rec = LogRecode {
            timestamp: "2026-11-11T12:13:45.123456".to_string(),
            level: LogLevel::INFO,
            msg: "hello".to_string(),
            pid: 1234,
        };
        assert_eq!(rec.to_log(), "2026-11-11T12:13:45.123456 [INFO] hello");
    }

    #[test]
    fn test_logger_basic_logging() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("app.log");

        {
            let mut logger = Logger::new(&log_path, LogLevel::INFO, 7, 1024, false).unwrap();
            logger.info("info message").unwrap();
            logger.debug("debug message").unwrap(); // Should be filtered
            logger.error("error message").unwrap();
            logger.out().unwrap();
        }

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("[INFO] info message"));
        assert!(!content.contains("[DEBUG] debug message"));
        assert!(content.contains("[ERROR] error message"));
    }

    #[test]
    fn test_logger_realtime() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("realtime.log");

        {
            let mut logger = Logger::new(&log_path, LogLevel::INFO, 7, 1024, true).unwrap();
            logger.info("realtime message").unwrap();
        }

        let content = fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("[INFO] realtime message"));
    }

    #[test]
    fn test_logger_rotation() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("rotate.log");

        {
            // Small size to trigger rotation
            let mut logger = Logger::new(&log_path, LogLevel::INFO, 3, 50, false).unwrap();
            logger.info("msg1").unwrap();
            logger.info("msg2").unwrap();
            logger.out().unwrap(); // msg1, msg2 written. size ~50.

            logger.info("msg3").unwrap();
            logger.out().unwrap(); // msg3 written. size > 50. Rotate.
        }

        assert!(log_path.exists());
        assert!(log_path.with_extension("log.1").exists());
    }

    #[test]
    fn test_logger_max_file_num_one() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("one.log");

        {
            let mut logger = Logger::new(&log_path, LogLevel::INFO, 1, 20, false).unwrap();
            logger.info("msg1").unwrap(); // ~25 bytes
            logger.out().unwrap(); // writes msg1, size > 20. Rotates.
        }

        // If max_file_num = 1, it should be truncated or app.log.1 should be deleted.
        // In my current implementation, it truncates.
        assert!(log_path.exists());
        let metadata = fs::metadata(&log_path).unwrap();
        assert_eq!(metadata.len(), 0);
    }
}
