use sql_logger::{SqlLogLevel, SqlLogRecord, SqlLogger};
use std::fs;

fn setup_logger(path: &str) -> SqlLogger {
    let _ = fs::remove_file(path);
    SqlLogger::new(path, SqlLogLevel::Debug).unwrap()
}

#[test]
fn test_log_level_display() {
    assert_eq!(format!("{}", SqlLogLevel::Info), "INFO");
    assert_eq!(format!("{}", SqlLogLevel::Debug), "DEBUG");
    assert_eq!(format!("{}", SqlLogLevel::Warning), "WARNING");
    assert_eq!(format!("{}", SqlLogLevel::Error), "ERROR");
    assert_eq!(format!("{}", SqlLogLevel::Critical), "CRITICAL");
}

#[test]
fn test_log_level_comparison() {
    assert!(SqlLogLevel::Critical > SqlLogLevel::Error);
    assert!(SqlLogLevel::Error > SqlLogLevel::Warning);
    assert!(SqlLogLevel::Warning > SqlLogLevel::Info);
    assert!(SqlLogLevel::Info > SqlLogLevel::Debug);
    assert!(SqlLogLevel::Critical > SqlLogLevel::Debug);
}

#[test]
fn test_logging_and_retrieval() {
    let path = "test_logs.db";
    let mut logger = setup_logger(path);

    logger.info("test_name", "info message").unwrap();
    logger.error("test_name", "error message").unwrap();
    logger.flush().unwrap();

    let recs: Vec<SqlLogRecord> = logger.get_recs().unwrap();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].name, "test_name");
    assert_eq!(recs[0].level, SqlLogLevel::Error);
    assert_eq!(recs[1].level, SqlLogLevel::Info);
}

#[test]
fn test_log_level_filtering() {
    let path = "test_logs_filter.db";
    let _ = setup_logger(path);

    // Set configured level to Warning
    let mut logger = SqlLogger::new(path, SqlLogLevel::Warning).unwrap();

    logger.info("test", "should be ignored").unwrap();
    logger.warning("test", "should be kept").unwrap();
    logger.error("test", "should be kept").unwrap();
    logger.flush().unwrap();

    let recs = logger.get_recs().unwrap();
    assert_eq!(recs.len(), 2);
    assert!(recs.iter().all(|r| r.level >= SqlLogLevel::Warning));
}

#[test]
fn test_get_recs_with() {
    let path = "test_logs_with.db";
    let mut logger = setup_logger(path);

    logger.info("app", "info 1").unwrap();
    logger.info("db", "db 1").unwrap();
    logger.error("app", "error 1").unwrap();
    logger.flush().unwrap();

    // By name
    let app_recs = logger.get_recs_with(Some("app"), None, None).unwrap();
    assert_eq!(app_recs.len(), 2);

    // By level
    let error_recs = logger
        .get_recs_with(None, Some(SqlLogLevel::Error), None)
        .unwrap();
    assert_eq!(error_recs.len(), 1);
    assert_eq!(error_recs[0].level, SqlLogLevel::Error);

    // By name and level
    let app_error_recs = logger
        .get_recs_with(Some("app"), Some(SqlLogLevel::Error), None)
        .unwrap();
    assert_eq!(app_error_recs.len(), 1);
    assert_eq!(app_error_recs[0].name, "app");

    // With limit
    let limited_recs = logger.get_recs_with(None, None, Some(1)).unwrap();
    assert_eq!(limited_recs.len(), 1);
}

#[test]
fn test_reduce() {
    let path = "test_logs_reduce.db";
    let mut logger = setup_logger(path);

    for i in 0..15 {
        logger.info("test", &format!("msg {}", i)).unwrap();
    }
    logger.flush().unwrap();

    // Reduce to 10
    logger.reduce(10, 30).unwrap();

    let recs = logger.get_recs().unwrap();
    assert_eq!(recs.len(), 10);
}

#[test]
fn test_initialize() {
    let path = "test_logs_init.db";
    {
        let mut logger = setup_logger(path);
        logger.info("test", "msg").unwrap();
        logger.flush().unwrap();
    }

    // Re-initialize
    let mut logger = SqlLogger::new(path, SqlLogLevel::Debug).unwrap();
    logger.initialize().unwrap();

    let recs = logger.get_recs().unwrap();
    assert_eq!(recs.len(), 0);
}

#[test]
fn test_realtime_mode() {
    let path = "test_logs_realtime.db";
    let mut logger = setup_logger(path);
    logger.set_realtime(true);

    logger.info("test", "realtime msg").unwrap();
    // In realtime mode, it should be written immediately without flush
    let recs = logger.get_recs().unwrap();
    assert_eq!(recs.len(), 1);
}

#[test]
fn test_buffered_mode() {
    let path = "test_logs_buffered.db";
    let mut logger = setup_logger(path);
    logger.set_realtime(false);

    logger.info("test", "buffered msg").unwrap();
    // Should not be in DB yet (assuming batch size 10)
    let recs = logger.get_recs().unwrap();
    assert_eq!(recs.len(), 0);

    logger.flush().unwrap();
    let recs = logger.get_recs().unwrap();
    assert_eq!(recs.len(), 1);
}
