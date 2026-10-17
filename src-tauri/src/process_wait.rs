//! Floe's bounded wait for actual process termination, without escalation.
use std::time::Duration;
pub fn wait_for_exit(check: impl FnMut() -> Result<bool, String>) -> Result<(), String> {
    poll_exit(
        check,
        || std::thread::sleep(Duration::from_millis(100)),
        100,
    )
}
fn poll_exit(
    mut check: impl FnMut() -> Result<bool, String>,
    mut pause: impl FnMut(),
    attempts: usize,
) -> Result<(), String> {
    for _ in 0..attempts {
        if check()? {
            return Ok(());
        }
        pause();
    }
    if check()? {
        Ok(())
    } else {
        Err("Application is still running, so it was not restarted".into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn an_exited_app_does_not_wait() {
        poll_exit(|| Ok(true), || panic!("unexpected wait"), 100).unwrap();
    }
    #[test]
    fn waits_for_exit_without_escalation() {
        let mut polls = 0;
        let mut pauses = 0;
        poll_exit(
            || {
                polls += 1;
                Ok(polls == 4)
            },
            || pauses += 1,
            100,
        )
        .unwrap();
        assert_eq!(pauses, 3);
    }
    #[test]
    fn a_refused_quit_stops_after_the_reference_deadline() {
        let mut pauses = 0;
        assert!(
            poll_exit(|| Ok(false), || pauses += 1, 100)
                .unwrap_err()
                .contains("not restarted")
        );
        assert_eq!(pauses, 100);
    }
    #[test]
    fn an_unreadable_process_state_never_counts_as_exit() {
        assert_eq!(
            poll_exit(
                || Err("unavailable".into()),
                || panic!("unexpected wait"),
                100
            )
            .unwrap_err(),
            "unavailable"
        );
    }
}
