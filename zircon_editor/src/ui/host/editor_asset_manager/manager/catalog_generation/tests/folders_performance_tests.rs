use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState};
use zircon_runtime::asset::{AssetId, AssetKind, AssetUri, AssetUuid};

use super::{
    build_folder_records, folder_display_name, ordered_child_folder_ids, terminal_folder_id,
    AssetCatalogRecord,
};

#[test]
fn folder_sort_borrows_display_names() {
    let source = include_str!("../folders.rs");
    let cloned_sort_key = ["asset_names.get(", ").cloned().unwrap_or_default()"].concat();
    let collected_segments = ["asset_path.split('/')", ".collect::<Vec<_>>()"].concat();

    assert!(!source.contains(&cloned_sort_key));
    assert!(!source.contains(&collected_segments));
}

#[test]
fn optimization_wave_20260824j_editor04_folder_child_hash_admission_preserves_order() {
    let mut child_folder_ids = HashSet::new();
    assert!(child_folder_ids.insert("res://zulu".to_string()));
    assert!(child_folder_ids.insert("res://beta".to_string()));
    assert!(child_folder_ids.insert("res://alpha".to_string()));
    assert!(!child_folder_ids.insert("res://beta".to_string()));

    assert_eq!(
        ordered_child_folder_ids(child_folder_ids),
        vec![
            "res://alpha".to_string(),
            "res://beta".to_string(),
            "res://zulu".to_string(),
        ]
    );

    let package_children = HashSet::from([
        "package://example/assets/中文".to_string(),
        "package://example/assets/Beta".to_string(),
        "package://example/assets/alpha".to_string(),
    ]);
    assert_eq!(
        ordered_child_folder_ids(package_children),
        vec![
            "package://example/assets/Beta".to_string(),
            "package://example/assets/alpha".to_string(),
            "package://example/assets/中文".to_string(),
        ]
    );
}

#[test]
fn optimization_wave_20260824j_editor04_folder_child_hash_admission_uses_set() {
    const SOURCE: &str = include_str!("../folders.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("child_folder_ids: HashSet<String>"));
    assert!(production.contains("child_folder_ids.insert(folder_id.clone())"));
    assert!(!production.contains("child_folder_ids.contains(&folder_id)"));
    assert!(!production.contains("child_folder_ids.push(folder_id.clone())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_wave_20260824j_editor04_folder_child_hash_admission_evidence() {
    const CANDIDATE_COUNT: usize = 4_096;
    const FOLDER_ID_BYTES: usize = 256;
    const LEGACY_LINEAR_COMPARISONS: usize = 8_386_560;
    const SAMPLE_COUNT: usize = 21;
    let suffix = "x".repeat(FOLDER_ID_BYTES - 15);
    let candidates = (0..CANDIDATE_COUNT)
        .map(|index| format!("res://{index:08}-{suffix}"))
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_child_folder_ids(black_box(&candidates)),
        || hashed_child_folder_ids(black_box(&candidates)),
    );
    assert_eq!(
        legacy_child_folder_ids(&candidates),
        hashed_child_folder_ids(&candidates)
    );

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR04_FOLDER_CHILD_HASH_ADMISSION_BENCH_V1 candidates={CANDIDATE_COUNT} folder_id_bytes={FOLDER_ID_BYTES} samples={SAMPLE_COUNT} sample_order=alternating legacy_linear_comparisons={LEGACY_LINEAR_COMPARISONS} optimized_hash_admissions={CANDIDATE_COUNT} deterministic_admission_reduction_percent=99.9512 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
fn optimization_wave_20260824j_editor04_folder_terminal_cache_preserves_ids() {
    assert_eq!(
        terminal_folder_id("res://", "characters/hero"),
        Some("res://characters/hero".to_string())
    );
    assert_eq!(
        terminal_folder_id("package://com.zircon.demo", "characters/hero"),
        Some("package://com.zircon.demo/characters/hero".to_string())
    );
    assert_eq!(terminal_folder_id("res://", ""), None);
}

#[test]
fn optimization_wave_20260824j_editor04_folder_terminal_cache_uses_existing_folder() {
    const SOURCE: &str = include_str!("../folders.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("terminal_folder_id(&root_id, folder_path)"));
    assert!(production.contains("folders.get_mut(&terminal_folder_id)"));
    let terminal_hit = production
        .split("if let Some(folder) = folders.get_mut(&terminal_folder_id)")
        .nth(1)
        .and_then(|body| body.split("let mut parent_id = root_id;").next())
        .expect("terminal-folder fast path");
    assert!(terminal_hit.contains("folder.recursive_asset_count += 1;"));
    assert!(terminal_hit.contains("continue;"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_wave_20260824j_editor04_folder_terminal_cache_evidence() {
    const ASSET_COUNT: usize = 4_096;
    const PATH_DEPTH: usize = 8;
    const LEGACY_SEGMENT_PATH_BUILDS: usize = ASSET_COUNT * PATH_DEPTH;
    const OPTIMIZED_PATH_BUILDS: usize = ASSET_COUNT + PATH_DEPTH;
    const SAMPLE_COUNT: usize = 21;
    let folder_path = (0..PATH_DEPTH)
        .map(|index| format!("segment-{index:02}-{}", "x".repeat(52)))
        .collect::<Vec<_>>()
        .join("/");

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_folder_path_resolution(black_box(&folder_path), ASSET_COUNT),
        || cached_folder_path_resolution(black_box(&folder_path), ASSET_COUNT),
    );
    assert_eq!(
        legacy_folder_path_resolution(&folder_path, ASSET_COUNT),
        cached_folder_path_resolution(&folder_path, ASSET_COUNT)
    );

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR04_FOLDER_TERMINAL_CACHE_BENCH_V1 assets={ASSET_COUNT} path_depth={PATH_DEPTH} samples={SAMPLE_COUNT} sample_order=alternating legacy_segment_path_builds={LEGACY_SEGMENT_PATH_BUILDS} optimized_path_builds={OPTIMIZED_PATH_BUILDS} deterministic_path_build_reduction_percent=87.4756 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
    );
}

#[test]
#[ignore = "Windows Release folder projection evidence"]
fn editor04_folder_sort_and_large_catalog_release_benchmark() {
    const ASSET_COUNT: usize = 4_096;
    const SAMPLE_COUNT: usize = 21;
    let parent_id = format!("res://{}", "deep/".repeat(8).trim_end_matches('/'));
    let candidates = (0..ASSET_COUNT)
        .map(|index| format!("{parent_id}/folder_{index:05}"))
        .collect::<HashSet<_>>();
    let catalog = large_folder_catalog(&parent_id, ASSET_COUNT);
    assert_eq!(
        legacy_folder_name_sort(candidates.clone()),
        ordered_child_folder_ids(candidates.clone())
    );
    let projected = build_folder_records(&catalog);
    let parent = projected
        .iter()
        .find(|folder| folder.folder_id == parent_id)
        .expect("large catalog parent folder");
    assert_eq!(parent.child_folder_ids.len(), ASSET_COUNT);
    assert_eq!(parent.recursive_asset_count, ASSET_COUNT);
    black_box(legacy_folder_name_sort(candidates.clone()));
    black_box(ordered_child_folder_ids(candidates.clone()));
    black_box(build_folder_records(&catalog));

    let mut legacy_sort_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_sort_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut catalog_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_sort_samples.push(measure_folder_sort(&candidates, legacy_folder_name_sort));
            optimized_sort_samples.push(measure_folder_sort(&candidates, ordered_child_folder_ids));
        } else {
            optimized_sort_samples.push(measure_folder_sort(&candidates, ordered_child_folder_ids));
            legacy_sort_samples.push(measure_folder_sort(&candidates, legacy_folder_name_sort));
        }
        let started = Instant::now();
        black_box(build_folder_records(black_box(&catalog)));
        catalog_samples.push(started.elapsed().as_nanos().max(1));
    }

    let legacy_p50 = percentile(&legacy_sort_samples, 50);
    let legacy_p95 = percentile(&legacy_sort_samples, 95);
    let legacy_p99 = percentile(&legacy_sort_samples, 99);
    let optimized_p50 = percentile(&optimized_sort_samples, 50);
    let optimized_p95 = percentile(&optimized_sort_samples, 95);
    let optimized_p99 = percentile(&optimized_sort_samples, 99);
    let catalog_p50 = percentile(&catalog_samples, 50);
    let catalog_p95 = percentile(&catalog_samples, 95);
    let catalog_p99 = percentile(&catalog_samples, 99);
    println!(
        "PERF_RESULT EDITOR04_FOLDER_SORT_LARGE_CATALOG_BENCH_V1 assets={ASSET_COUNT} sibling_folders={ASSET_COUNT} samples={SAMPLE_COUNT} legacy_sort_p50_ns={legacy_p50} legacy_sort_p95_ns={legacy_p95} legacy_sort_p99_ns={legacy_p99} optimized_sort_p50_ns={optimized_p50} optimized_sort_p95_ns={optimized_p95} optimized_sort_p99_ns={optimized_p99} catalog_build_p50_ns={catalog_p50} catalog_build_p95_ns={catalog_p95} catalog_build_p99_ns={catalog_p99} threshold_sort_p95_ratio=0.70"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
        "borrowed folder sort p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
    );
}

fn large_folder_catalog(
    parent_id: &str,
    asset_count: usize,
) -> std::collections::HashMap<AssetUuid, AssetCatalogRecord> {
    (0..asset_count)
        .map(|index| {
            let uuid = AssetUuid::from_stable_label(&format!("folder-asset-{index:05}"));
            let locator = AssetUri::parse(&format!(
                "{parent_id}/folder_{index:05}/asset_{index:05}.png"
            ))
            .expect("catalog fixture locator");
            let record = AssetCatalogRecord {
                asset_uuid: uuid,
                asset_id: AssetId::from_asset_uuid(uuid),
                locator: locator.clone(),
                kind: AssetKind::Texture,
                display_name: format!("Asset {index:05}"),
                file_name: format!("asset_{index:05}.png"),
                extension: "png".to_string(),
                meta_path: Default::default(),
                meta: AssetMetaDocument::new(uuid, locator, AssetKind::Texture),
                source_mtime_unix_ms: 0,
                source_hash: String::new(),
                preview_state: PreviewState::Dirty,
                preview_artifact_path: Default::default(),
                dirty: false,
                diagnostics: Vec::new(),
                direct_references: Vec::new(),
            };
            (uuid, record)
        })
        .collect()
}

fn legacy_folder_name_sort(child_folder_ids: HashSet<String>) -> Vec<String> {
    let folder_names = child_folder_ids
        .iter()
        .map(|folder_id| (folder_id.clone(), folder_display_name(folder_id).to_owned()))
        .collect::<std::collections::HashMap<_, _>>();
    let mut child_folder_ids = child_folder_ids.into_iter().collect::<Vec<_>>();
    child_folder_ids.sort_by(|left, right| {
        folder_names[left]
            .cmp(&folder_names[right])
            .then(left.cmp(right))
    });
    child_folder_ids
}

fn measure_folder_sort(
    candidates: &HashSet<String>,
    sort: fn(HashSet<String>) -> Vec<String>,
) -> u128 {
    let candidates = candidates.clone();
    let started = Instant::now();
    black_box(sort(black_box(candidates)));
    started.elapsed().as_nanos().max(1)
}

fn legacy_child_folder_ids(candidates: &[String]) -> Vec<String> {
    let mut child_folder_ids = Vec::with_capacity(candidates.len());
    for folder_id in candidates {
        if !child_folder_ids.contains(folder_id) {
            child_folder_ids.push(folder_id.clone());
        }
    }
    child_folder_ids.sort_unstable();
    child_folder_ids
}

fn hashed_child_folder_ids(candidates: &[String]) -> Vec<String> {
    let mut child_folder_ids = candidates.iter().cloned().collect::<HashSet<_>>();
    let mut child_folder_ids = child_folder_ids.drain().collect::<Vec<_>>();
    child_folder_ids.sort_unstable();
    child_folder_ids
}

fn legacy_folder_path_resolution(folder_path: &str, asset_count: usize) -> Vec<String> {
    let mut folders = BTreeSet::new();
    for _ in 0..asset_count {
        let mut parent_id = "res://".to_string();
        for segment in folder_path.split('/') {
            let folder_id = if parent_id == "res://" {
                format!("res://{segment}")
            } else {
                format!("{parent_id}/{segment}")
            };
            let _ = folders.insert(folder_id.clone());
            parent_id = folder_id;
        }
    }
    folders.into_iter().collect()
}

fn cached_folder_path_resolution(folder_path: &str, asset_count: usize) -> Vec<String> {
    let mut folders = BTreeSet::new();
    for _ in 0..asset_count {
        let terminal_folder_id = format!("res://{folder_path}");
        if folders.contains(&terminal_folder_id) {
            continue;
        }
        let mut parent_id = "res://".to_string();
        for segment in folder_path.split('/') {
            let folder_id = if parent_id == "res://" {
                format!("res://{segment}")
            } else {
                format!("{parent_id}/{segment}")
            };
            let _ = folders.insert(folder_id.clone());
            parent_id = folder_id;
        }
    }
    folders.into_iter().collect()
}

fn benchmark_paired_samples<const SAMPLE_COUNT: usize>(
    mut legacy: impl FnMut() -> Vec<String>,
    mut optimized: impl FnMut() -> Vec<String>,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> Vec<String>) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}
