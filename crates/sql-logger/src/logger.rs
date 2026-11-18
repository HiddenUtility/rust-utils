use crate::error::SqlLoggerError;
use crate::level::SqlLogLevel;
use crate::record::SqlLogRecord;
use rusqlite::{params, Connection};

pub struct SqlLogger {
    connection: Connection,
    configured_level: SqlLogLevel,
    buffer: Vec<SqlLogRecord>,
    realtime: bool,
    max_rec_num: usize,
    max_rec_days: u32,
}

impl SqlLogger {
    pub fn new(db_path: &str, configured_level: SqlLogLevel) -> Result<Self, SqlLoggerError> {
        let connection = Connection::open(db_path)?;
        let mut logger = Self {
            connection,
            configured_level,
            buffer: Vec::new(),
            realtime: false,
            max_rec_num: 1000,
            max_rec_days: 30,
        };
        logger.initialize()?;
        Ok(logger)
    }

    pub fn new_with<F>(
        db_path: &str,
        configured_level: SqlLogLevel,
        initializer: F,
    ) -> Result<Self, SqlLoggerError>
    where
        F: FnOnce(&Connection) -> Result<(), SqlLoggerError>,
    {
        let connection = Connection::open(db_path)?;
        initializer(&connection)?;
        let logger = Self {
            connection,
            configured_level,
            buffer: Vec::new(),
            realtime: false,
            max_rec_num: 1000,
            max_rec_days: 30,
        };
        Ok(logger)
    }

    pub fn set_realtime(&mut self, realtime: bool) {
        self.realtime = realtime;
    }

    pub fn set_limits(&mut self, max_rec_num: usize, max_rec_days: u32) {
        self.max_rec_num = max_rec_num;
        self.max_rec_days = max_rec_days;
    }

    pub fn out(&mut self, record: SqlLogRecord) -> Result<(), SqlLoggerError> {
        if record.level < self.configured_level {
            return Ok(());
        }

        if self.realtime {
            self.insert_record(&record)?;
        } else {
            self.buffer.push(record);
            if self.buffer.len() >= 10 {
                self.flush()?;
            }
        }
        Ok(())
    }

    fn insert_record(&mut self, record: &SqlLogRecord) -> Result<(), SqlLoggerError> {
        self.connection.execute(
            "INSERT INTO sql_logs (timestamp, level, name, message) VALUES (?1, ?2, ?3, ?4)",
            params![
                record.timestamp,
                record.level.to_string(),
                record.name,
                record.message
            ],
        )?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), SqlLoggerError> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        let tx = self.connection.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO sql_logs (timestamp, level, name, message) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for record in self.buffer.drain(..) {
                stmt.execute(params![
                    record.timestamp,
                    record.level.to_string(),
                    record.name,
                    record.message
                ])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_recs(&self) -> Result<Vec<SqlLogRecord>, SqlLoggerError> {
        self.get_recs_with(None, None, None)
    }

    pub fn get_recs_with_limit(&self, limit: usize) -> Result<Vec<SqlLogRecord>, SqlLoggerError> {
        self.get_recs_with(None, None, Some(limit))
    }

    pub fn get_recs_with(
        &self,
        name: Option<&str>,
        level: Option<SqlLogLevel>,
        limit: Option<usize>,
    ) -> Result<Vec<SqlLogRecord>, SqlLoggerError> {
        let mut query =
            "SELECT id, timestamp, level, name, message FROM sql_logs WHERE 1=1".to_string();
        let mut params_vec: Vec<String> = Vec::new();

        if let Some(n) = name {
            query.push_str(" AND name = ?");
            params_vec.push(n.to_string());
        }

        if let Some(l) = level {
            query.push_str(" AND level = ?");
            params_vec.push(l.to_string());
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }

        let mut stmt = self.connection.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            let id: i64 = row.get(0)?;
            let timestamp: i64 = row.get(1)?;
            let level_str: String = row.get(2)?;
            let name: String = row.get(3)?;
            let message: String = row.get(4)?;

            let level = match level_str.as_str() {
                "DEBUG" => SqlLogLevel::Debug,
                "INFO" => SqlLogLevel::Info,
                "WARNING" => SqlLogLevel::Warning,
                "ERROR" => SqlLogLevel::Error,
                "CRITICAL" => SqlLogLevel::Critical,
                _ => SqlLogLevel::Info,
            };

            Ok(SqlLogRecord {
                id: Some(id),
                timestamp,
                level,
                name,
                message,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn reduce(&mut self, max_rec_num: usize, max_rec_days: u32) -> Result<(), SqlLoggerError> {
        let count: usize = self
            .connection
            .query_row("SELECT COUNT(*) FROM sql_logs", [], |r| r.get(0))?;

        if count <= max_rec_num {
            return Ok(());
        }

        let cutoff_timestamp =
            chrono::Utc::now().timestamp() - (max_rec_days as i64 * 24 * 60 * 60);
        self.connection.execute(
            "DELETE FROM sql_logs WHERE timestamp < ?1",
            params![cutoff_timestamp],
        )?;

        let new_count: usize =
            self.connection
                .query_row("SELECT COUNT(*) FROM sql_logs", [], |r| r.get(0))?;
        if new_count > max_rec_num {
            let to_delete = new_count - max_rec_num;
            self.connection.execute(
                "DELETE FROM sql_logs WHERE id IN (SELECT id FROM sql_logs ORDER BY timestamp ASC, id ASC LIMIT ?1)",
                params![to_delete],
            )?;
        }

        Ok(())
    }

    pub fn initialize(&mut self) -> Result<(), SqlLoggerError> {
        self.connection
            .execute("DROP TABLE IF EXISTS sql_logs", [])?;
        self.connection.execute(
            "CREATE TABLE sql_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                level TEXT NOT NULL,
                name TEXT NOT NULL,
                message TEXT NOT NULL
            )",
            [],
        )?;
        self.connection.execute(
            "CREATE INDEX idx_sql_logs_timestamp ON sql_logs (timestamp)",
            [],
        )?;
        self.connection
            .execute("CREATE INDEX idx_sql_logs_name ON sql_logs (name)", [])?;
        self.connection
            .execute("CREATE INDEX idx_sql_logs_level ON sql_logs (level)", [])?;
        Ok(())
    }

    pub fn critical(&mut self, name: &str, message: &str) -> Result<(), SqlLoggerError> {
        self.out(SqlLogRecord::new(SqlLogLevel::Critical, name, message))
    }

    pub fn error(&mut self, name: &str, message: &str) -> Result<(), SqlLoggerError> {
        self.out(SqlLogRecord::new(SqlLogLevel::Error, name, message))
    }

    pub fn warning(&mut self, name: &str, message: &str) -> Result<(), SqlLoggerError> {
        self.out(SqlLogRecord::new(SqlLogLevel::Warning, name, message))
    }

    pub fn info(&mut self, name: &str, message: &str) -> Result<(), SqlLoggerError> {
        self.out(SqlLogRecord::new(SqlLogLevel::Info, name, message))
    }

    pub fn debug(&mut self, name: &str, message: &str) -> Result<(), SqlLoggerError> {
        self.out(SqlLogRecord::new(SqlLogLevel::Debug, name, message))
    }
}

impl Drop for SqlLogger {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
