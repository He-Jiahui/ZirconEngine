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
    runtime_asset_path_from_roots(relative.as_ref(), std::iter::empty())
}

pub fn runtime_asset_path_with_dev_asset_root(
    relative: impl AsRef<Path>,
    dev_asset_root: impl AsRef<Path>,
) -> PathBuf {
    runtime_asset_path_from_roots(
        relative.as_ref(),
        std::iter::once(dev_asset_root.as_ref().to_path_buf()),
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
            let (exists, is_dir) = root_status(&candidate);
            diagnostics::write_verbose(format!(
                "selected_authoritative_root path={} exists={} is_dir={}",
                candidate.display(),
                exists,
                is_dir
            ));
        }
        return candidate;
    }
    for candidate in candidates.paths {
        let (exists, is_dir) = root_status(&candidate);
        if diagnostics::verbose_enabled() {
            diagnostics::write_verbose(format!(
                "root_candidate path={} exists={} is_dir={}",
                candidate.display(),
                exists,
                is_dir
            ));
        }
        if is_dir {
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

fn runtime_asset_path_from_roots(
    path: &Path,
    dev_asset_roots: impl IntoIterator<Item = PathBuf>,
) -> PathBuf {
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
                .map(|candidate| {
                    let (exists, is_dir) = root_status(candidate);
                    format!("{}|exists={exists}|dir={is_dir}", candidate.display())
                })
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
        if !root_status(&candidate).1 {
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

/// Snapshot both diagnostic flags and admission from one filesystem metadata lookup.
fn root_status(path: &Path) -> (bool, bool) {
    match path.metadata() {
        Ok(metadata) => (true, metadata.is_dir()),
        Err(_) => (false, false),
    }
}

struct RuntimeAssetRootCandidates {
    paths: Vec<PathBuf>,
    authoritative: bool,
}

fn runtime_asset_root_candidates() -> RuntimeAssetRootCandidates {
    runtime_asset_root_candidates_with_dev_roots(std::iter::empty())
}

fn runtime_asset_root_candidates_with_dev_roots(
    dev_asset_roots: impl IntoIterator<Item = PathBuf>,
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
    dev_asset_roots: impl IntoIterator<Item = PathBuf>,
    explicit_root: Option<&OsStr>,
    executable: Option<&Path>,
) -> RuntimeAssetRootCandidates {
    let dev_asset_roots = dev_asset_roots.into_iter();
    let mut candidates = Vec::with_capacity(dev_asset_roots.size_hint().0.saturating_add(2));

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

    let crate_root = crate_asset_root();
    if !candidates.iter().any(|candidate| candidate == &crate_root) {
        candidates.push(crate_root);
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
#[path = "tests/runtime_asset_path.rs"]
mod tests;
