//! 尽力补全当前目录并词法折叠路径；Windows 统一大小写，供路径 gate registry 作为键使用。
use std::path::{Path, PathBuf};

// 相对路径尝试补当前目录，再消除 . 与 ..；获取目录失败时仍对输入做词法折叠。
pub(super) fn absolute_path(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|directory| directory.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalize_platform_path(normalized)
}

#[cfg(windows)]
fn normalize_platform_path(path: PathBuf) -> PathBuf {
    PathBuf::from(path.to_string_lossy().to_lowercase())
}

#[cfg(not(windows))]
fn normalize_platform_path(path: PathBuf) -> PathBuf {
    path
}
