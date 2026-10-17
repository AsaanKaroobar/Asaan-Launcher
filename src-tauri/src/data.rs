//! Same platform data resolver for resident and headless commands. No directory creation.
pub const IDENTIFIER: &str = "com.asaaan.launcher";
pub fn personalization_path() -> Result<std::path::PathBuf, String> {
    dirs::data_dir()
        .map(|path| path.join(IDENTIFIER).join("personalization.json"))
        .ok_or_else(|| "Application data directory is unavailable".into())
}
#[cfg(test)]
mod tests {
    #[test]
    fn data_identity_matches_bundled_configuration() {
        let config: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(config["identifier"], super::IDENTIFIER);
        assert!(
            config["app"].get("appDirectoriesOverride").is_none(),
            "Update the shared data resolver when adding directory overrides"
        );
    }
}
