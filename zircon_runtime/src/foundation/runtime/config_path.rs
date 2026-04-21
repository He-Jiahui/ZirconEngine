use std::path::PathBuf;

pub(super) fn config_file_path() -> PathBuf {
    if let Some(path) = std::env::var_os("ZIRCON_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if cfg!(target_os = "windows") {
        if let Some(base) = std::env::var_os("LOCALAPPDATA").or_else(|| std::env::var_os("APPDATA"))
        {
            return PathBuf::from(base).join("ZirconEngine").join("config.json");
        }
    } else if let Some(base) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(base).join("ZirconEngine").join("config.json");
    } else if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("ZirconEngine")
            .join("config.json");
    }

    PathBuf::from(".zircon-config.json")
}
