//! Bounded private clipboard request contract. No clipboard history or text logging.
pub const MAX_TEXT: usize = 64 * 1024;
pub const TIMEOUT_MS: u64 = 5000;
pub fn validate(text: &str, deadline: u64, now: u64) -> Result<std::time::Duration, String> {
    if text.is_empty() || text.len() > MAX_TEXT || text.contains('\0') {
        return Err("Clipboard text is empty, invalid or too large".into());
    }
    if deadline <= now || deadline.saturating_sub(now) > TIMEOUT_MS {
        return Err("Clipboard request expired or has an invalid deadline".into());
    }
    Ok(std::time::Duration::from_millis(deadline - now))
}
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_is_bounded_by_utf8_bytes_and_never_truncated() {
        assert!(validate(&"é".repeat(MAX_TEXT / 2), 110, 100).is_ok());
        assert!(validate(&"é".repeat(MAX_TEXT / 2 + 1), 110, 100).is_err());
        assert!(validate("", 110, 100).is_err());
        assert!(validate("x\0y", 110, 100).is_err());
    }
    #[test]
    fn stale_and_unbounded_deadlines_are_rejected() {
        assert!(validate("x", 100, 100).is_err());
        assert!(validate("x", 99, 100).is_err());
        assert!(validate("x", 5101, 100).is_err());
        assert_eq!(
            validate("x", 5100, 100).unwrap(),
            std::time::Duration::from_millis(5000)
        );
    }
}
