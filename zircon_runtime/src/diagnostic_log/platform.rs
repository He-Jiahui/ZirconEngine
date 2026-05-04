use std::path::{Path, PathBuf};

const LOG_ROOT_ENV: &str = "ZIRCON_LOG_ROOT";
const COMPANY_NAME: &str = "ZirconEngine";
const PRODUCT_NAME: &str = "ZirconEngine";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct LogDirectoryCandidate {
    pub source: &'static str,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticLogLocation {
    LocalFirst,
    UnityCompatibleFirst,
}

pub(super) fn log_directory_candidates(
    timestamp: &str,
    location: DiagnosticLogLocation,
) -> Vec<LogDirectoryCandidate> {
    let mut candidates = Vec::new();

    if let Some(root) = std::env::var_os(LOG_ROOT_ENV).filter(|value| !value.is_empty()) {
        candidates.push(LogDirectoryCandidate {
            source: LOG_ROOT_ENV,
            path: PathBuf::from(root).join(timestamp),
        });
    }

    if matches!(location, DiagnosticLogLocation::UnityCompatibleFirst) {
        push_unity_compatible_candidate(&mut candidates, timestamp);
    }

    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
    {
        candidates.push(LogDirectoryCandidate {
            source: "executable-directory",
            path: log_directory_under_root(&exe_dir, timestamp),
        });
    }

    if let Ok(current_dir) = std::env::current_dir() {
        let path = log_directory_under_root(&current_dir, timestamp);
        if !candidates.iter().any(|candidate| candidate.path == path) {
            candidates.push(LogDirectoryCandidate {
                source: "current-directory",
                path,
            });
        }
    }

    if matches!(location, DiagnosticLogLocation::LocalFirst) {
        push_unity_compatible_candidate(&mut candidates, timestamp);
    }

    candidates
}

fn push_unity_compatible_candidate(candidates: &mut Vec<LogDirectoryCandidate>, timestamp: &str) {
    if let Some(path) = unity_compatible_log_directory(timestamp) {
        if !candidates.iter().any(|candidate| candidate.path == path) {
            candidates.push(LogDirectoryCandidate {
                source: "unity-compatible-user-log-directory",
                path,
            });
        }
    }
}

fn log_directory_under_root(root: &Path, timestamp: &str) -> PathBuf {
    root.join("logs").join(timestamp)
}

fn unity_compatible_log_directory(timestamp: &str) -> Option<PathBuf> {
    unity_compatible_log_root().map(|root| root.join("logs").join(timestamp))
}

#[cfg(target_os = "windows")]
fn unity_compatible_log_root() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|home| {
            home.join("AppData")
                .join("LocalLow")
                .join(COMPANY_NAME)
                .join(PRODUCT_NAME)
        })
}

#[cfg(target_os = "macos")]
fn unity_compatible_log_root() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|home| {
            home.join("Library")
                .join("Logs")
                .join(COMPANY_NAME)
                .join(PRODUCT_NAME)
        })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn unity_compatible_log_root() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|home| {
            home.join(".config")
                .join("unity3d")
                .join(COMPANY_NAME)
                .join(PRODUCT_NAME)
        })
}

#[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
fn unity_compatible_log_root() -> Option<PathBuf> {
    None
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::log_directory_under_root;

    #[test]
    fn log_directory_uses_logs_timestamp_under_root() {
        let path = log_directory_under_root(Path::new("engine"), "2026-05-04-12-30-45");

        assert!(path.ends_with(Path::new("engine/logs/2026-05-04-12-30-45")));
    }
}
