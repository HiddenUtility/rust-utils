use crate::level::SqlLogLevel;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SqlLogRecord {
    pub id: Option<i64>,
    pub timestamp: i64, // Unix timestamp
    pub level: SqlLogLevel,
    pub name: String,
    pub message: String,
}

impl SqlLogRecord {
    pub fn new(level: SqlLogLevel, name: &str, message: &str) -> Self {
        Self {
            id: None,
            timestamp: Utc::now().timestamp(),
            level,
            name: name.to_string(),
            message: message.to_string(),
        }
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        Utc.timestamp_opt(self.timestamp, 0)
            .single()
            .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap())
    }
}

impl fmt::Display for SqlLogRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} - {}: {}",
            self.datetime().to_rfc3339(),
            self.level,
            self.name,
            self.message
        )
    }
}

impl fmt::Debug for SqlLogRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SqlLogRecord {{ id: {:?}, timestamp: \"{}\", level: {}, name: \"{}\", message: \"{}\" }}",
            self.id,
            self.datetime().to_rfc3339(),
            self.level,
            self.name,
            self.message
        )
    }
}
