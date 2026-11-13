use std::time::SystemTime;

/// 現在時刻を取得します。
pub fn get_now() -> Result<SystemTime, String> {
    Ok(SystemTime::now())
}

#[cfg(test)]
mod tests {
    use super::get_now;
    use std::time::SystemTime;

    #[test]
    fn test_get_now_should_return_recent_time() {
        let now_result = get_now();
        let now = now_result.expect("get_now failed");
        let now_ms = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let current_ms = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Check that the difference is less than 1 second
        assert!((now_ms as i128 - current_ms as i128).abs() < 1000);
    }
}
