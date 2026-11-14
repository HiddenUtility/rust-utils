use logger::LogLevel;

#[test]
fn test_log_level_display() {
    assert_eq!(format!("{}", LogLevel::CRITICAL), "CRITICAL");
    assert_eq!(format!("{}", LogLevel::ERROR), "ERROR");
    assert_eq!(format!("{}", LogLevel::WARNING), "WARNING");
    assert_eq!(format!("{}", LogLevel::INFO), "INFO");
    assert_eq!(format!("{}", LogLevel::DEBUG), "DEBUG");
}
