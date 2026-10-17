//! Settings process policy adapted from Floe SettingsProcess.swift / SettingsMode.swift.
//! Copyright (Floe) © 2026 René Jiménez. GNU AGPLv3.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Page {
    #[default]
    General,
    Applications,
}
impl Page {
    pub fn id(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Applications => "applications",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "general" => Some(Self::General),
            "applications" => Some(Self::Applications),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    Start(Page),
    Forward(Option<Page>),
}
pub fn step(running: bool, page: Option<Page>, last: Option<Page>) -> Step {
    if running {
        Step::Forward(page)
    } else {
        Step::Start(page.or(last).unwrap_or_default())
    }
}
pub fn exit_due(window_open: bool, work_pending: bool) -> bool {
    !window_open && !work_pending
}
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub requested: bool,
    pub page: Option<Page>,
}
pub fn options(args: &[String]) -> Result<Options, String> {
    let mut result = Options::default();
    let mut iter = args.iter();
    let mut named = false;
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--settings" => {
                if result.requested {
                    return Err("Repeated --settings".into());
                }
                result.requested = true;
            }
            "--page" => {
                if named {
                    return Err("Repeated --page".into());
                }
                named = true;
                result.page = Some(
                    iter.next()
                        .and_then(|s| Page::parse(s))
                        .ok_or("Settings page must be general or applications")?,
                );
            }
            s if s.starts_with("--page=")
                || s.starts_with("--settings=")
                || s == "--extension"
                || s.starts_with("--extension=") =>
            {
                return Err("Unsupported settings option".into());
            }
            _ => {}
        }
    }
    if named && !result.requested {
        return Err("--page requires --settings".into());
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opening_settings_starts_when_none_runs() {
        assert_eq!(step(false, None, None), Step::Start(Page::General));
        assert_eq!(
            step(false, Some(Page::Applications), None),
            Step::Start(Page::Applications)
        );
    }
    #[test]
    fn a_new_process_opens_where_last_left() {
        assert_eq!(
            step(false, None, Some(Page::Applications)),
            Step::Start(Page::Applications)
        );
        assert_eq!(
            step(false, Some(Page::General), Some(Page::Applications)),
            Step::Start(Page::General)
        );
    }
    #[test]
    fn opening_again_brings_running_process_forward() {
        assert_eq!(
            step(true, None, Some(Page::Applications)),
            Step::Forward(None)
        );
        assert_eq!(
            step(true, Some(Page::General), Some(Page::Applications)),
            Step::Forward(Some(Page::General))
        );
    }
    #[test]
    fn mode_parses_supported_pages() {
        let a = |s: &[&str]| s.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            options(&a(&["--settings"])).unwrap(),
            Options {
                requested: true,
                page: None
            }
        );
        assert_eq!(
            options(&a(&["--settings", "--page", "applications"]))
                .unwrap()
                .page,
            Some(Page::Applications)
        );
        assert!(!options(&[]).unwrap().requested);
    }
    #[test]
    fn a_page_needs_settings_and_a_real_name() {
        for args in [
            vec!["--page", "general"],
            vec!["--settings", "--page", "nowhere"],
            vec!["--settings", "--page"],
            vec!["--settings", "--extension", "planets"],
            vec!["--settings=true"],
            vec!["--settings", "--settings"],
        ] {
            assert!(options(&args.iter().map(|s| s.to_string()).collect::<Vec<_>>()).is_err());
        }
    }
    #[test]
    fn every_supported_page_reads_back() {
        for page in [Page::General, Page::Applications] {
            assert_eq!(Page::parse(page.id()), Some(page));
        }
        for invalid in ["", "privacy", "extension:"] {
            assert_eq!(Page::parse(invalid), None);
        }
    }
    #[test]
    fn exits_only_after_window_closes_and_work_finishes() {
        assert!(exit_due(false, false));
        assert!(!exit_due(true, false));
        assert!(!exit_due(false, true));
        assert!(!exit_due(true, true));
    }
}
