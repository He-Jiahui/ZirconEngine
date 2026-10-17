//! 最近项目文档、home 路径和跨进程写锁 sidecar 共用的路径规则。
use std::path::{Path, PathBuf};

const HUB_RECENT_PROJECTS_FILE_NAME: &str = "recent_projects.json";

/// Returns the shared Hub and Editor registry path required by protocol v1.
/// 返回协议 v1 的最近项目文件路径；缺少用户主目录环境变量时退回相对 `.zircon/hub` 路径。
/// 返回宿主配置根目录下的共享最近项目文档路径。
pub fn hub_recent_projects_path() -> PathBuf {
    let home = if cfg!(target_os = "windows") {
        std::env::var_os("USERPROFILE")
    } else {
        std::env::var_os("HOME")
    };
    home.map(|home| hub_recent_projects_path_from_home(PathBuf::from(home)))
        .unwrap_or_else(|| {
            PathBuf::from(".zircon")
                .join("hub")
                .join(HUB_RECENT_PROJECTS_FILE_NAME)
        })
}

/// Builds the protocol v1 path from an explicit home directory for deterministic host tests.
/// 使用显式 home 根目录构造最近项目文档路径，便于宿主隔离配置位置。
pub fn hub_recent_projects_path_from_home(home: impl AsRef<Path>) -> PathBuf {
    home.as_ref()
        .join(".zircon")
        .join("hub")
        .join(HUB_RECENT_PROJECTS_FILE_NAME)
}

/// Sidecar file used by non-Windows hosts for a short exclusive read-merge-write lease.
/// 返回非 Windows 主机用于独占写事务的 sidecar 锁路径。
pub fn hub_recent_projects_lock_path(recent_projects_path: impl AsRef<Path>) -> PathBuf {
    let recent_projects_path = recent_projects_path.as_ref();
    let file_name = recent_projects_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(HUB_RECENT_PROJECTS_FILE_NAME);
    recent_projects_path.with_file_name(format!(".{file_name}.lock"))
}
