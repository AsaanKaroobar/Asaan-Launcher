//! Floe Model.panelDidHide/panelWillShow use a 90-second default command resume window.
use std::time::Duration;
pub const DELAY: Duration = Duration::from_secs(90);
#[derive(Default)]
pub struct Resume {
    visible: bool,
    expires: Option<Duration>,
}
impl Resume {
    pub fn visible(&mut self, visible: bool, active: bool, now: Duration) {
        self.visible = visible;
        self.expires = if visible || !active {
            None
        } else {
            Some(now + DELAY)
        };
    }
    pub fn started(&mut self, now: Duration) {
        self.expires = if self.visible {
            None
        } else {
            Some(now + DELAY)
        };
    }
    pub fn ended(&mut self) {
        self.expires = None;
    }
    pub fn expired(&self, now: Duration) -> bool {
        self.expires.is_some_and(|at| now >= at)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_hidden_command_survives_until_the_ninety_second_boundary() {
        let mut r = Resume::default();
        r.visible(true, true, Duration::ZERO);
        r.started(Duration::ZERO);
        assert!(!r.expired(Duration::from_secs(500)));
        r.visible(false, true, Duration::from_secs(500));
        assert!(!r.expired(Duration::from_secs(589)));
        assert!(r.expired(Duration::from_secs(590)));
    }
    #[test]
    fn showing_cancels_reset_and_hiding_again_starts_a_new_window() {
        let mut r = Resume::default();
        r.started(Duration::ZERO);
        r.visible(true, true, Duration::from_secs(89));
        assert!(!r.expired(Duration::from_secs(100)));
        r.visible(false, true, Duration::from_secs(100));
        assert!(!r.expired(Duration::from_secs(189)));
        assert!(r.expired(Duration::from_secs(190)));
        r.ended();
        assert!(!r.expired(Duration::from_secs(500)));
    }
    #[test]
    fn hiding_without_a_command_does_not_schedule_a_reset() {
        let mut r = Resume::default();
        r.visible(false, false, Duration::ZERO);
        assert!(!r.expired(Duration::from_secs(100)));
    }
}
