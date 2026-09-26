use std::path::{Path, PathBuf};

pub const PROJECT_SESSION_LOCK_FILE_NAME: &str = "session.lock";

/// 派生持久记录位置；文件存在本身不证明仍有活跃的平台租约。
pub fn project_session_lock_path(project_root: impl AsRef<Path>) -> PathBuf {
    project_root
        .as_ref()
        .join(".zircon")
        .join(PROJECT_SESSION_LOCK_FILE_NAME)
}

/// 按传入根路径的 UTF-16 原样散列互斥名；调用方必须先选择同一物理根的规范形式。
// TODO: [CR-PROJECT-0004] Hub 与 Editor 各自规范化根路径；补跨端等价测试，覆盖扩展前缀与非 Unicode 路径。
#[cfg(windows)]
pub fn windows_project_session_mutex_name(project_root: impl AsRef<Path>) -> String {
    use std::os::windows::ffi::OsStrExt;

    let mut bytes = Vec::new();
    for unit in project_root.as_ref().as_os_str().encode_wide() {
        bytes.extend(unit.to_le_bytes());
    }
    format!(
        "Global\\ZirconEngineProjectSession-{}",
        blake3::hash(&bytes).to_hex()
    )
}
