use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use crate::asset::project::ProjectPaths;

#[cfg(feature = "diagnostic-log")]
#[path = "runtime_asset_path/diagnostics_enabled.rs"]
mod diagnostics;
#[cfg(not(feature = "diagnostic-log"))]
#[path = "runtime_asset_path/diagnostics_disabled.rs"]
mod diagnostics;

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
    let candidates = runtime_asset_root_candidates();
    if candidates.authoritative {
        let candidate = candidates
            .paths
            .into_iter()
            .next()
            .expect("authoritative runtime asset root should be resolved");
        if diagnostics::verbose_enabled() {
            diagnostics::write_verbose(format!(
                "selected_authoritative_root path={} exists={} is_dir={}",
                candidate.display(),
                candidate.exists(),
                candidate.is_dir()
            ));
        }
        return candidate;
    }
    for candidate in candidates.paths {
        if diagnostics::verbose_enabled() {
            diagnostics::write_verbose(format!(
                "root_candidate path={} exists={} is_dir={}",
                candidate.display(),
                candidate.exists(),
                candidate.is_dir()
            ));
        }
        if candidate.exists() && candidate.is_dir() {
            if diagnostics::verbose_enabled() {
                diagnostics::write_verbose(format!("selected_root path={}", candidate.display()));
            }
            return candidate;
        }
    }
    let fallback = crate_asset_root();
    if diagnostics::verbose_enabled() {
        diagnostics::write_verbose(format!(
            "selected_root_fallback path={}",
            fallback.display()
        ));
    }
    fallback
}

fn runtime_asset_path_from_roots(path: &Path, dev_asset_roots: Vec<PathBuf>) -> PathBuf {
    let relative = normalize_runtime_asset_relative_path(path);
    let candidates = runtime_asset_root_candidates_with_dev_roots(dev_asset_roots);
    runtime_asset_path_from_candidates(&relative, candidates)
}

fn runtime_asset_path_from_candidates(
    relative: &Path,
    candidates: RuntimeAssetRootCandidates,
) -> PathBuf {
    if diagnostics::verbose_enabled() {
        diagnostics::write_verbose(format!(
            "resolve normalized={} authoritative={} candidates={}",
            relative.display(),
            candidates.authoritative,
            candidates
                .paths
                .iter()
                .map(|candidate| format!(
                    "{}|exists={}|dir={}",
                    candidate.display(),
                    candidate.exists(),
                    candidate.is_dir()
                ))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    if candidates.authoritative {
        let candidate = candidates
            .paths
            .into_iter()
            .next()
            .expect("authoritative runtime asset root should be resolved");
        let resolved = candidate.join(relative);
        if diagnostics::verbose_enabled() {
            diagnostics::write_verbose(format!(
                "resolved_authoritative relative={} selected_root={} path={} path_exists={}",
                relative.display(),
                candidate.display(),
                resolved.display(),
                resolved.exists()
            ));
        }
        return resolved;
    }
    for candidate in candidates.paths {
        if !(candidate.exists() && candidate.is_dir()) {
            continue;
        }
        let resolved = candidate.join(&relative);
        if diagnostics::verbose_enabled() {
            diagnostics::write_verbose(format!(
                "resolved relative={} selected_root={} path={} path_exists={}",
                relative.display(),
                candidate.display(),
                resolved.display(),
                resolved.exists()
            ));
        }
        return resolved;
    }
    let fallback_root = crate_asset_root();
    let resolved = fallback_root.join(&relative);
    if diagnostics::verbose_enabled() {
        diagnostics::write_verbose(format!(
            "resolved_fallback relative={} selected_root={} path={} path_exists={}",
            relative.display(),
            fallback_root.display(),
            resolved.display(),
            resolved.exists()
        ));
    }
    resolved
}

struct RuntimeAssetRootCandidates {
    paths: Vec<PathBuf>,
    authoritative: bool,
}

fn runtime_asset_root_candidates() -> RuntimeAssetRootCandidates {
    runtime_asset_root_candidates_with_dev_roots(Vec::new())
}

fn runtime_asset_root_candidates_with_dev_roots(
    dev_asset_roots: Vec<PathBuf>,
) -> RuntimeAssetRootCandidates {
    let executable = std::env::current_exe().ok();
    let explicit_root = std::env::var_os(ZIRCON_ASSET_ROOT_ENV);
    runtime_asset_root_candidates_with_inputs(
        dev_asset_roots,
        explicit_root.as_deref(),
        executable.as_deref(),
    )
}

fn runtime_asset_root_candidates_with_inputs(
    dev_asset_roots: Vec<PathBuf>,
    explicit_root: Option<&OsStr>,
    executable: Option<&Path>,
) -> RuntimeAssetRootCandidates {
    let mut candidates = Vec::new();

    if let Some(root) = explicit_root {
        let root = Path::new(root);
        if !root.as_os_str().is_empty()
            && !root.to_str().is_some_and(|value| value.trim().is_empty())
        {
            let root = resolve_environment_asset_root(root, executable).unwrap_or_else(|| {
                panic!(
                    "{ZIRCON_ASSET_ROOT_ENV} must be absolute or resolvable from the product executable"
                )
            });
            candidates.push(root);
            // A declared product root is authoritative even when the requested asset is absent.
            return RuntimeAssetRootCandidates {
                paths: candidates,
                authoritative: true,
            };
        }
    }

    if let Some(executable) = executable {
        if let Some(root) = default_product_asset_root_from_executable(executable) {
            candidates.push(root);
        }
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
    RuntimeAssetRootCandidates {
        paths: candidates,
        authoritative: false,
    }
}

/// Resolves an asset-root environment override without giving it an implicit working directory.
///
/// Relative values describe the staged product layout beside the executable. The returned path is
/// an operation path selected by the shared resolver, preserving aliases without introducing an
/// additional virtual-path scheme for engine assets.
fn resolve_environment_asset_root(root: &Path, executable: Option<&Path>) -> Option<PathBuf> {
    if root.as_os_str().is_empty() || root.to_str().is_some_and(|value| value.trim().is_empty()) {
        return None;
    }
    if root.is_absolute() {
        return ProjectPaths::resolve_path(root)
            .ok()
            .map(|root| root.into_operation_path());
    }
    let product_directory = executable?.parent()?;
    let product_directory = ProjectPaths::resolve_path(product_directory).ok()?;
    ProjectPaths::resolve_path_from(&product_directory, root)
        .ok()
        .map(|root| root.into_operation_path())
}

/// Resolves the product's conventional asset directory from the executable identity.
///
/// This keeps the default staged layout on the same resolver path as an explicit relative
/// `ZIRCON_ASSET_ROOT`, without adding a second distribution-path convention.
fn default_product_asset_root_from_executable(executable: &Path) -> Option<PathBuf> {
    resolve_environment_asset_root(Path::new("assets"), Some(executable))
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
    use std::path::Path;

    use crate::asset::project::ProjectPaths;

    use super::{
        default_product_asset_root_from_executable, resolve_environment_asset_root,
        runtime_asset_path, runtime_asset_path_from_candidates,
        runtime_asset_path_with_dev_asset_root, runtime_asset_root_candidates_with_inputs,
    };

    #[test]
    fn explicit_environment_asset_root_is_the_only_product_candidate() {
        let root = std::env::temp_dir().join(format!(
            "zircon-runtime-explicit-asset-root-{}",
            std::process::id()
        ));
        let product_directory = root.join("product");
        let current_directory = root.join("project");
        let dev_asset_root = root.join("source-assets");
        std::fs::create_dir_all(&product_directory).unwrap();
        std::fs::create_dir_all(current_directory.join("assets")).unwrap();
        std::fs::create_dir_all(&dev_asset_root).unwrap();
        let executable = product_directory.join("zircon_runtime.exe");

        let candidates = runtime_asset_root_candidates_with_inputs(
            vec![dev_asset_root],
            Some(std::ffi::OsStr::new("assets")),
            Some(&executable),
        );

        assert!(candidates.authoritative);
        assert_eq!(candidates.paths.len(), 1);
        assert_eq!(
            candidates.paths[0],
            ProjectPaths::resolve_path(product_directory.join("assets"))
                .unwrap()
                .into_operation_path()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[should_panic(expected = "must be absolute or resolvable from the product executable")]
    fn relative_environment_asset_root_without_product_identity_fails_closed() {
        let _ = runtime_asset_root_candidates_with_inputs(
            Vec::new(),
            Some(std::ffi::OsStr::new("assets")),
            None,
        );
    }

    #[test]
    fn project_working_directory_is_not_an_engine_asset_root_candidate() {
        let root = std::env::temp_dir().join(format!(
            "zircon-runtime-engine-root-boundary-{}",
            std::process::id()
        ));
        let executable = root.join("product").join("zircon_editor.exe");
        let dev_asset_root = root.join("source").join("zircon_editor").join("assets");
        let project_asset_root = root.join("project").join("assets");

        let candidates = runtime_asset_root_candidates_with_inputs(
            vec![dev_asset_root.clone()],
            None,
            Some(&executable),
        );

        assert!(!candidates.authoritative);
        assert_eq!(
            candidates.paths,
            vec![
                ProjectPaths::resolve_path(root.join("product").join("assets"))
                    .unwrap()
                    .into_operation_path(),
                dev_asset_root,
                super::crate_asset_root(),
            ]
        );
        assert!(!candidates.paths.contains(&project_asset_root));
    }

    #[test]
    fn missing_product_asset_does_not_fall_back_to_a_development_root() {
        let root = std::env::temp_dir().join(format!(
            "zircon-runtime-missing-product-asset-{}",
            std::process::id()
        ));
        let product_asset_root = root.join("product").join("assets");
        let dev_asset_root = root.join("source-assets");
        let relative = Path::new("ui/editor/host/editor_main_frame.zui");
        std::fs::create_dir_all(&product_asset_root).unwrap();
        std::fs::create_dir_all(dev_asset_root.join(relative).parent().unwrap()).unwrap();
        std::fs::write(dev_asset_root.join(relative), b"development-only fixture").unwrap();

        let resolved = runtime_asset_path_from_candidates(
            relative,
            super::RuntimeAssetRootCandidates {
                paths: vec![product_asset_root.clone(), dev_asset_root],
                authoritative: true,
            },
        );

        assert_eq!(resolved, product_asset_root.join(relative));
        assert!(!resolved.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn selected_existing_root_does_not_change_per_requested_asset() {
        let root = std::env::temp_dir().join(format!(
            "zircon-runtime-stable-asset-root-{}",
            std::process::id()
        ));
        let product_asset_root = root.join("product").join("assets");
        let dev_asset_root = root.join("source-assets");
        let relative = Path::new("icons/only-in-source.svg");
        std::fs::create_dir_all(&product_asset_root).unwrap();
        std::fs::create_dir_all(dev_asset_root.join(relative).parent().unwrap()).unwrap();
        std::fs::write(dev_asset_root.join(relative), b"development-only fixture").unwrap();

        let resolved = runtime_asset_path_from_candidates(
            relative,
            super::RuntimeAssetRootCandidates {
                paths: vec![product_asset_root.clone(), dev_asset_root],
                authoritative: false,
            },
        );

        assert_eq!(resolved, product_asset_root.join(relative));
        assert!(!resolved.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relative_environment_asset_root_uses_the_product_executable_directory() {
        let root = std::env::temp_dir().join(format!(
            "zircon-runtime-asset-root-relative-{}",
            std::process::id()
        ));
        let product_directory = root.join("product");
        std::fs::create_dir_all(&product_directory).unwrap();
        let executable = product_directory.join("zircon_runtime.exe");

        let resolved = resolve_environment_asset_root(Path::new("assets"), Some(&executable))
            .expect("a relative asset-root override should resolve from the product directory");
        let expected = ProjectPaths::resolve_path(product_directory.join("assets"))
            .unwrap()
            .into_operation_path();

        assert_eq!(resolved, expected);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn default_product_asset_root_uses_the_resolved_executable_directory() {
        let root = std::env::temp_dir().join(format!(
            "zircon-runtime-default-asset-root-{}",
            std::process::id()
        ));
        let product_directory = root.join("product");
        std::fs::create_dir_all(&product_directory).unwrap();
        let executable = product_directory.join("zircon_runtime.exe");

        let resolved = default_product_asset_root_from_executable(&executable)
            .expect("a product executable should provide a default asset root");
        let expected = ProjectPaths::resolve_path(product_directory.join("assets"))
            .unwrap()
            .into_operation_path();

        assert_eq!(resolved, expected);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn absolute_environment_asset_root_remains_an_external_override() {
        let root = std::env::temp_dir().join(format!(
            "zircon-runtime-asset-root-absolute-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();

        let resolved = resolve_environment_asset_root(&root, None)
            .expect("an absolute asset-root override should remain supported");
        let expected = ProjectPaths::resolve_path(&root)
            .unwrap()
            .into_operation_path();

        assert_eq!(resolved, expected);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relative_environment_asset_root_without_an_executable_is_not_cwd_relative() {
        assert_eq!(
            resolve_environment_asset_root(Path::new("assets"), None),
            None
        );
    }

    #[test]
    fn runtime_asset_path_accepts_paths_with_or_without_assets_prefix() {
        let direct = runtime_asset_path("ui/runtime/fixtures/hud_overlay.zui");
        let prefixed = runtime_asset_path("assets/ui/runtime/fixtures/hud_overlay.zui");
        let rooted = runtime_asset_path("/assets/ui/runtime/fixtures/hud_overlay.zui");

        assert_eq!(direct, prefixed);
        assert_eq!(direct, rooted);
        assert!(
            direct.ends_with("ui/runtime/fixtures/hud_overlay.zui"),
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
        let expected = dev_root.join("ui/editor/editor_main_frame.zui");
        std::fs::create_dir_all(expected.parent().unwrap()).unwrap();
        std::fs::write(&expected, b"fixture").unwrap();

        let resolved = runtime_asset_path_with_dev_asset_root(
            "assets/ui/editor/editor_main_frame.zui",
            &dev_root,
        );

        let _ = std::fs::remove_dir_all(&dev_root);
        assert_eq!(resolved, expected);
    }
}
