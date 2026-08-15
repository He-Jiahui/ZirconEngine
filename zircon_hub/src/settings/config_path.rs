use std::path::PathBuf;

pub fn default_hub_config_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Some(base) = std::env::var_os("LOCALAPPDATA").or_else(|| std::env::var_os("APPDATA"))
        {
            return PathBuf::from(base).join("ZirconHub").join("config.toml");
        }
    } else if let Some(base) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(base).join("ZirconHub").join("config.toml");
    } else if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("ZirconHub")
            .join("config.toml");
    }

    PathBuf::from(".zircon-hub.toml")
}
