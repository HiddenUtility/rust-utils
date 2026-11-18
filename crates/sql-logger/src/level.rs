use serde::{Deserialize, Serialize};
use std::fmt;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SqlLogLevel {
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
}

impl fmt::Display for SqlLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SqlLogLevel::Debug => "DEBUG",
            SqlLogLevel::Info => "INFO",
            SqlLogLevel::Warning => "WARNING",
            SqlLogLevel::Error => "ERROR",
            SqlLogLevel::Critical => "CRITICAL",
        };
        write!(f, "{}", s)
    }
}
