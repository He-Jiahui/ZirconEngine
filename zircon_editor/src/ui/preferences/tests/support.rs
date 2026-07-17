use std::path::PathBuf;

pub(super) fn temp_appearance_preferences_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-editor-appearance-preferences-{}-{}-{}.toml",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ))
}
