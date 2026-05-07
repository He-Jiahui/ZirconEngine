use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use crate::diagnostic_log::{diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel};

const ZIRCON_ASSET_ROOT_ENV: &str = "ZIRCON_ASSET_ROOT";

pub fn runtime_asset_path(relative: impl AsRef<Path>) -> PathBuf {
    runtime_asset_path_from_roots(relative.as_ref(), Vec::new())
}

pub fn runtime_asset_path_with_dev_asset_root(
    relative: impl AsRef<Path>,
    dev_asset_root: impl AsRef<Path>,
) -> PathBuf {
    runtime_asset_path_from_roots(
        relative.as_ref(),
        vec![dev_asset_root.as_ref().to_path_buf()],
    )
}

pub fn runtime_asset_root() -> PathBuf {
    for candidate in runtime_asset_root_candidates() {
        if runtime_asset_path_verbose_enabled() {
            write_diagnostic_log(
                "runtime_asset_path",
                format!(
                    "root_candidate path={} exists={} is_dir={}",
                    candidate.display(),
                    candidate.exists(),
                    candidate.is_dir()
                ),
            );
        }
        if candidate.exists() && candidate.is_dir() {
            if runtime_asset_path_verbose_enabled() {
                write_diagnostic_log(
                    "runtime_asset_path",
                    format!("selected_root path={}", candidate.display()),
                );
            }
            return candidate;
        }
    }
    let fallback = crate_asset_root();
    if runtime_asset_path_verbose_enabled() {
        write_diagnostic_log(
            "runtime_asset_path",
            format!("selected_root_fallback path={}", fallback.display()),
        );
    }
    fallback
}

fn runtime_asset_path_from_roots(path: &Path, dev_asset_roots: Vec<PathBuf>) -> PathBuf {
    let relative = normalize_runtime_asset_relative_path(path);
    let candidates = runtime_asset_root_candidates_with_dev_roots(dev_asset_roots);
    if runtime_asset_path_verbose_enabled() {
        write_diagnostic_log(
            "runtime_asset_path",
            format!(
                "resolve relative={} normalized={} candidates={}",
                path.display(),
                relative.display(),
                candidates
                    .iter()
                    .map(|candidate| format!(
                        "{}|exists={}|dir={}",
                        candidate.display(),
                        candidate.exists(),
                        candidate.is_dir()
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        );
    }
    let mut first_existing_root = None;
    for candidate in candidates {
        if !(candidate.exists() && candidate.is_dir()) {
            continue;
        }
        let resolved = candidate.join(&relative);
        if first_existing_root.is_none() {
            first_existing_root = Some((candidate.clone(), resolved.clone()));
        }
        if resolved.exists() {
            if runtime_asset_path_verbose_enabled() {
                write_diagnostic_log(
                    "runtime_asset_path",
                    format!(
                        "resolved relative={} selected_root={} path={} path_exists={}",
                        relative.display(),
                        candidate.display(),
                        resolved.display(),
                        resolved.exists()
                    ),
                );
            }
            return resolved;
        }
    }
    if let Some((candidate, resolved)) = first_existing_root {
        if runtime_asset_path_verbose_enabled() {
            write_diagnostic_log(
                "runtime_asset_path",
                format!(
                    "resolved_missing relative={} selected_root={} path={} path_exists=false",
                    relative.display(),
                    candidate.display(),
                    resolved.display()
                ),
            );
        }
        return resolved;
    }
    let fallback_root = crate_asset_root();
    let resolved = fallback_root.join(&relative);
    if runtime_asset_path_verbose_enabled() {
        write_diagnostic_log(
            "runtime_asset_path",
            format!(
                "resolved_fallback relative={} selected_root={} path={} path_exists={}",
                relative.display(),
                fallback_root.display(),
                resolved.display(),
                resolved.exists()
            ),
        );
    }
    resolved
}

fn runtime_asset_path_verbose_enabled() -> bool {
    diagnostic_log_allows(DiagnosticLogLevel::Verbose)
}

fn runtime_asset_root_candidates() -> Vec<PathBuf> {
    runtime_asset_root_candidates_with_dev_roots(Vec::new())
}

fn runtime_asset_root_candidates_with_dev_roots(dev_asset_roots: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(root) = std::env::var_os(ZIRCON_ASSET_ROOT_ENV) {
        candidates.push(PathBuf::from(root));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("assets"));
        }
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("assets"));
    }

    for root in dev_asset_roots {
        if !candidates.iter().any(|candidate| candidate == &root) {
            candidates.push(root);
        }
    }

    if !candidates
        .iter()
        .any(|candidate| candidate == &crate_asset_root())
    {
        candidates.push(crate_asset_root());
    }
    candidates
}

fn crate_asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn normalize_runtime_asset_relative_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_)
            | Component::RootDir
            | Component::CurDir
            | Component::ParentDir => {}
            Component::Normal(value)
                if normalized.as_os_str().is_empty() && value == OsStr::new("assets") => {}
            Component::Normal(value) => normalized.push(value),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::{runtime_asset_path, runtime_asset_path_with_dev_asset_root};

    #[test]
    fn runtime_asset_path_accepts_paths_with_or_without_assets_prefix() {
        let direct = runtime_asset_path("ui/runtime/fixtures/hud_overlay.ui.toml");
        let prefixed = runtime_asset_path("assets/ui/runtime/fixtures/hud_overlay.ui.toml");
        let rooted = runtime_asset_path("/assets/ui/runtime/fixtures/hud_overlay.ui.toml");

        assert_eq!(direct, prefixed);
        assert_eq!(direct, rooted);
        assert!(
            direct.ends_with("ui/runtime/fixtures/hud_overlay.ui.toml"),
            "unexpected runtime asset path: {}",
            direct.display()
        );
    }

    #[test]
    fn runtime_asset_path_can_use_a_call_site_dev_asset_root() {
        let dev_root = std::env::temp_dir().join(format!(
            "zircon_runtime_asset_path_dev_root_{}",
            std::process::id()
        ));
        let expected = dev_root.join("ui/editor/editor_main_frame.ui.toml");
        std::fs::create_dir_all(expected.parent().unwrap()).unwrap();
        std::fs::write(&expected, b"fixture").unwrap();

        let resolved = runtime_asset_path_with_dev_asset_root(
            "assets/ui/editor/editor_main_frame.ui.toml",
            &dev_root,
        );

        let _ = std::fs::remove_dir_all(&dev_root);
        assert_eq!(resolved, expected);
    }
}
