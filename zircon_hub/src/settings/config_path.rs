use std::path::PathBuf;

/// 仅为首次启动或迁移缺省配置选择用户级存储路径；运行时始终由 HubConfig 负责读写。
/// 未发现用户配置目录时退回相对路径，此时实际位置受进程工作目录影响。
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
