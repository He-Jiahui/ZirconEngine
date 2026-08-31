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
mod tests {
    use std::hint::black_box;
    use std::path::Path;
    use std::time::Instant;

    use crate::asset::project::ProjectPaths;

    use super::{
        default_product_asset_root_from_executable, resolve_environment_asset_root,
        runtime_asset_path, runtime_asset_path_from_candidates,
        runtime_asset_path_with_dev_asset_root, runtime_asset_root_candidates_with_inputs,
    };

    const SAMPLE_PAIRS: usize = 21;
    const CANDIDATE_BUILDS_PER_SAMPLE: usize = 16_384;

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
    fn single_dev_root_iterator_preserves_candidate_order_and_deduplication() {
        let dev_asset_root = std::env::temp_dir().join("zircon-runtime-single-dev-root");

        let candidates = runtime_asset_root_candidates_with_inputs(
            std::iter::once(dev_asset_root.clone()),
            None,
            None,
        );

        assert!(!candidates.authoritative);
        assert_eq!(
            candidates.paths,
            vec![dev_asset_root, super::crate_asset_root()]
        );
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

    #[test]
    #[ignore = "release-only performance contract"]
    fn benchmark_single_dev_root_candidate_construction() {
        let dev_asset_root = std::env::temp_dir().join("zircon-runtime-candidate-benchmark");
        let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            if pair_index % 2 == 0 {
                legacy_raw.push(measure_candidate_builds(
                    legacy_single_dev_root_candidates,
                    &dev_asset_root,
                ));
                optimized_raw.push(measure_candidate_builds(
                    optimized_single_dev_root_candidates,
                    &dev_asset_root,
                ));
            } else {
                optimized_raw.push(measure_candidate_builds(
                    optimized_single_dev_root_candidates,
                    &dev_asset_root,
                ));
                legacy_raw.push(measure_candidate_builds(
                    legacy_single_dev_root_candidates,
                    &dev_asset_root,
                ));
            }
        }

        let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
        let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
        let improvement_percent = legacy_p95_ns
            .saturating_sub(optimized_p95_ns)
            .saturating_mul(100)
            / legacy_p95_ns.max(1);
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "iterator-backed candidate construction must improve P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
        println!(
            "PERF_RESULT task=plugins07_iterator_runtime_asset_roots sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank candidate_builds_per_sample={CANDIDATE_BUILDS_PER_SAMPLE} candidates_per_build=2 legacy_allocations_per_build=5 optimized_allocations_per_build=3 threshold_percent=15 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
    }

    fn legacy_single_dev_root_candidates(dev_asset_root: &Path) -> usize {
        let dev_asset_roots = vec![dev_asset_root.to_path_buf()];
        let mut candidates = Vec::new();
        for root in dev_asset_roots {
            if !candidates.iter().any(|candidate| candidate == &root) {
                candidates.push(root);
            }
        }
        if !candidates
            .iter()
            .any(|candidate| candidate == &super::crate_asset_root())
        {
            candidates.push(super::crate_asset_root());
        }
        black_box(candidate_checksum(&candidates))
    }

    fn optimized_single_dev_root_candidates(dev_asset_root: &Path) -> usize {
        let candidates = runtime_asset_root_candidates_with_inputs(
            std::iter::once(dev_asset_root.to_path_buf()),
            None,
            None,
        );
        black_box(candidate_checksum(&candidates.paths))
    }

    fn candidate_checksum(candidates: &[std::path::PathBuf]) -> usize {
        candidates
            .iter()
            .map(|candidate| candidate.components().count())
            .sum()
    }

    fn measure_candidate_builds(plan: fn(&Path) -> usize, dev_asset_root: &Path) -> u64 {
        let started = Instant::now();
        let mut checksum = 0;
        for _ in 0..CANDIDATE_BUILDS_PER_SAMPLE {
            checksum ^= plan(black_box(dev_asset_root));
        }
        black_box(checksum);
        u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw_samples(samples: &[u64]) -> String {
        samples
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
