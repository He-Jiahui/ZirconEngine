use std::hint::black_box;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::asset::project::ProjectPaths;

use super::{
    default_product_asset_root_from_executable, resolve_environment_asset_root, root_status,
    runtime_asset_path, runtime_asset_path_from_candidates, runtime_asset_path_with_dev_asset_root,
    runtime_asset_root_candidates_with_inputs, AssetRelativePath, AssetRelativePathError,
};

const SAMPLE_PAIRS: usize = 21;
const CANDIDATE_BUILDS_PER_SAMPLE: usize = 16_384;
const ROOT_PROBES_PER_SAMPLE: usize = 4_096;

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
        &AssetRelativePath::parse(relative).unwrap(),
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
        &AssetRelativePath::parse(relative).unwrap(),
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
fn root_probe_skips_missing_and_regular_file_candidates() {
    let root = std::env::temp_dir().join(format!(
        "zircon-runtime-root-probe-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let regular_file = root.join("not-a-root");
    let directory = root.join("assets");
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(&regular_file, b"fixture").unwrap();

    assert_eq!(root_status(&root.join("missing")), (false, false));
    assert_eq!(root_status(&regular_file), (true, false));
    assert_eq!(root_status(&directory), (true, true));
    assert_eq!(
        runtime_asset_path_from_candidates(
            &AssetRelativePath::parse("icons/test.svg").unwrap(),
            super::RuntimeAssetRootCandidates {
                paths: vec![root.join("missing"), regular_file, directory.clone()],
                authoritative: false,
            },
        ),
        directory.join("icons/test.svg")
    );
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
    let relative = AssetRelativePath::parse("ui/runtime/fixtures/hud_overlay.zui").unwrap();
    let legacy =
        AssetRelativePath::from_assets_prefixed("assets/ui/runtime/fixtures/hud_overlay.zui")
            .unwrap();
    let direct = runtime_asset_path(&relative);
    let prefixed = runtime_asset_path(&legacy);

    assert_eq!(direct, prefixed);
    assert_eq!(
        AssetRelativePath::parse("/assets/ui/runtime/fixtures/hud_overlay.zui"),
        Err(AssetRelativePathError::Rooted)
    );
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

    let relative =
        AssetRelativePath::from_assets_prefixed("assets/ui/editor/editor_main_frame.zui").unwrap();
    let resolved = runtime_asset_path_with_dev_asset_root(&relative, &dev_root);

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
        "PERF_RESULT task=plugins07_iterator_runtime_asset_roots sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank candidate_builds_per_sample={CANDIDATE_BUILDS_PER_SAMPLE} candidates_per_build=2 legacy_allocations_per_build_expected=5 optimized_allocations_per_build_expected=3 threshold_percent=15 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn benchmark_single_metadata_probe_for_existing_root() {
    let root = std::env::temp_dir().join(format!(
        "zircon-runtime-root-probe-bench-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    assert!(legacy_existing_root_probe(&root));
    assert!(single_existing_root_probe(&root));

    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_root_probes(&root, legacy_existing_root_probe));
            optimized_raw.push(measure_root_probes(&root, single_existing_root_probe));
        } else {
            optimized_raw.push(measure_root_probes(&root, single_existing_root_probe));
            legacy_raw.push(measure_root_probes(&root, legacy_existing_root_probe));
        }
    }
    std::fs::remove_dir_all(root).unwrap();

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    println!(
        "RUNTIME_ASSET_ROOT_SINGLE_METADATA_PROBE_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} sample_pairs={SAMPLE_PAIRS} probes_per_sample={ROOT_PROBES_PER_SAMPLE} existing_root_metadata_probes=2->1 legacy_raw_ns={} optimized_raw_ns={}",
        raw_samples(&legacy_raw),
        raw_samples(&optimized_raw)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(95),
        "single metadata probe P95 should be at most 95% of legacy"
    );
}

fn legacy_existing_root_probe(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

fn single_existing_root_probe(path: &Path) -> bool {
    root_status(path).1
}

fn measure_root_probes(path: &Path, probe: fn(&Path) -> bool) -> u64 {
    let started = Instant::now();
    let mut hits = 0;
    for _ in 0..ROOT_PROBES_PER_SAMPLE {
        if probe(black_box(path)) {
            hits += 1;
        }
    }
    assert_eq!(black_box(hits), ROOT_PROBES_PER_SAMPLE);
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
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

#[test]
fn strict_relative_path_preserves_valid_spelling() {
    for value in [
        "ui/runtime/fixtures/hud_overlay.zui",
        "assets/ui/icon.zui",
        "fonts/with space/default#100%.font.toml",
        "\u{6d4b}\u{8bd5}/\u{5b57}\u{4f53}.font.toml",
    ] {
        let relative = AssetRelativePath::parse(value).unwrap();
        assert_eq!(relative.as_path().as_os_str(), value);
    }
}

#[test]
fn strict_relative_path_rejects_root_prefix_and_traversal_components() {
    for value in [
        "",
        ".",
        "..",
        "./ui/icon.zui",
        "../ui/icon.zui",
        "ui/./icon.zui",
        "ui/../icon.zui",
        "ui/../../icon.zui",
        "/ui/icon.zui",
        "/assets/ui/icon.zui",
        "//server/share/icon.zui",
        "C:/ui/icon.zui",
        "C:ui/icon.zui",
        "ui/icon.zui:stream",
        r"\ui\icon.zui",
        r"\\server\share\icon.zui",
        r"\\?\C:\ui\icon.zui",
        r"ui\icon.zui",
        "ui//icon.zui",
        "ui/icon.zui/",
        "ui/\0icon.zui",
        "ui/\nicon.zui",
    ] {
        assert!(AssetRelativePath::parse(value).is_err(), "{value:?}");
    }
    assert_eq!(
        AssetRelativePath::parse("a/../b"),
        Err(AssetRelativePathError::ParentDirectory)
    );
}

#[test]
fn legacy_assets_prefix_is_explicit_and_stripped_only_once() {
    assert_eq!(
        AssetRelativePath::parse("assets/ui/icon.zui")
            .unwrap()
            .as_path(),
        Path::new("assets/ui/icon.zui")
    );
    assert_eq!(
        AssetRelativePath::from_assets_prefixed("assets/assets/ui/icon.zui")
            .unwrap()
            .as_path(),
        Path::new("assets/ui/icon.zui")
    );
    for value in [
        "ui/icon.zui",
        "/assets/ui/icon.zui",
        "assets/",
        "assets/../icon.zui",
        "assets/./icon.zui",
        "assets//icon.zui",
        r"assets/C:\icon.zui",
    ] {
        assert!(
            AssetRelativePath::from_assets_prefixed(value).is_err(),
            "{value:?}"
        );
    }
}

#[test]
fn asset_root_query_requires_the_explicit_root_value() {
    assert_eq!(
        AssetRelativePath::parse(""),
        Err(AssetRelativePathError::Empty)
    );
    assert!(AssetRelativePath::root().as_path().as_os_str().is_empty());
    assert_eq!(
        runtime_asset_path(&AssetRelativePath::root()),
        super::runtime_asset_root()
    );
}

#[cfg(unix)]
#[test]
fn strict_relative_path_rejects_non_utf8_spelling_on_unix() {
    use std::os::unix::ffi::OsStrExt;
    let path = Path::new(std::ffi::OsStr::from_bytes(b"ui/\xff.zui"));
    assert_eq!(
        AssetRelativePath::parse(path),
        Err(AssetRelativePathError::NonUtf8)
    );
}

#[cfg(windows)]
#[test]
fn strict_relative_path_rejects_non_utf8_spelling_on_windows() {
    use std::os::windows::ffi::OsStringExt;
    let value = std::ffi::OsString::from_wide(&[b'u' as u16, b'i' as u16, 47, 0xd800]);
    assert_eq!(
        AssetRelativePath::parse(Path::new(&value)),
        Err(AssetRelativePathError::NonUtf8)
    );
}
