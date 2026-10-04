use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::{
    collect_v2_sources, load_ui_v2_source_with_receipt, sha256_hex, UiV2FileStoreCacheKey,
    UiV2PrototypeStoreFileCache,
};

const SOURCE_COUNT: usize = 512;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bm_ui_v2_cache_canonical_revalidation_preserves_key() {
    let canonical = source_path();

    assert_eq!(
        UiV2FileStoreCacheKey::from_paths(std::slice::from_ref(&canonical)),
        UiV2FileStoreCacheKey::from_canonical_paths(std::slice::from_ref(&canonical))
    );
}

#[test]
fn source_key_invalidates_same_length_content_when_timestamp_is_preserved() {
    let root = std::env::temp_dir().join(format!(
        "zircon-ui-v2-source-fingerprint-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the epoch")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("temporary source directory should be created");
    let path = root.join("source.zui");
    std::fs::write(&path, b"first bytes").expect("initial bytes should be written");
    let modified = std::fs::metadata(&path)
        .expect("initial metadata should be available")
        .modified()
        .expect("initial modification time should be available");
    let first = UiV2FileStoreCacheKey::from_paths(std::slice::from_ref(&path));

    std::fs::write(&path, b"other bytes").expect("same-length replacement should be written");
    std::fs::File::open(&path)
        .expect("replacement should be readable")
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .expect("the original modification time should be restored");

    let metadata = std::fs::metadata(&path).expect("replacement metadata should be available");
    assert_eq!(metadata.len(), 11);
    assert_eq!(metadata.modified().unwrap(), modified);
    let second = UiV2FileStoreCacheKey::from_paths(std::slice::from_ref(&path));

    assert_ne!(
        first, second,
        "source bytes must participate in cache identity"
    );
    std::fs::remove_dir_all(root).expect("temporary source directory should be removed");
}

#[test]
fn parsed_source_receipt_hashes_the_exact_file_bytes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/runtime/fixtures/settings_dialog.zui");
    let raw_bytes = std::fs::read(&path).expect("built-in fixture bytes should be readable");

    let (document, receipt) =
        load_ui_v2_source_with_receipt(&path).expect("built-in fixture should parse");

    assert_eq!(receipt.asset_id, document.asset.id);
    assert_eq!(receipt.sha256, sha256_hex(&raw_bytes));
    assert_eq!(receipt.physical_path, path.canonicalize().unwrap());
}

#[test]
fn load_store_reloads_same_length_source_after_timestamp_is_restored() {
    let root = std::env::temp_dir().join(format!(
        "zircon-ui-v2-source-reload-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the epoch")
            .as_nanos()
    ));
    let source_root = root.join("assets/ui/runtime/fixtures");
    std::fs::create_dir_all(&source_root).expect("temporary asset directory should be created");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/runtime/fixtures/settings_dialog.zui");
    let source_bytes = std::fs::read(&source).expect("fixture source should be readable");
    let target = source_root.join("settings_dialog.zui");
    std::fs::write(&target, &source_bytes).expect("fixture copy should be written");

    let mut cache = UiV2PrototypeStoreFileCache::new();
    let first = cache
        .load_store(std::iter::once(&target))
        .expect("fixture should load into the V2 cache");
    let original_time = std::fs::metadata(&target)
        .expect("fixture metadata should be available")
        .modified()
        .expect("fixture modification time should be available");
    let first_hash = first.source_receipts[0].sha256.clone();

    let text = String::from_utf8(source_bytes).expect("fixture should be valid UTF-8");
    let replaced = text.replace("settings_dialog.zui", "settings_diolog.zui");
    assert_ne!(replaced, text, "fixture must contain its asset URI");
    assert_eq!(
        replaced.len(),
        text.len(),
        "replacement must preserve byte length"
    );
    std::fs::write(&target, replaced.as_bytes()).expect("same-length replacement should write");
    std::fs::File::open(&target)
        .expect("replacement should be readable")
        .set_times(std::fs::FileTimes::new().set_modified(original_time))
        .expect("original modification time should be restored");

    let second = cache
        .load_store(std::iter::once(&target))
        .expect("updated fixture should reload");
    assert!(
        !second.cache_hit,
        "changed source bytes must invalidate the cached entry"
    );
    assert_ne!(second.source_receipts[0].sha256, first_hash);
    assert!(second.root_asset_id.ends_with("settings_diolog.zui"));
    std::fs::remove_dir_all(root).expect("temporary asset directory should be removed");
}

#[test]
fn source_collection_retains_unresolved_asset_id_imports_for_audit() {
    let root = std::env::temp_dir().join(format!(
        "zircon-ui-v2-unresolved-import-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the epoch")
            .as_nanos()
    ));
    let source_root = root.join("assets/ui/runtime/fixtures");
    std::fs::create_dir_all(&source_root).expect("temporary asset directory should be created");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/ui/runtime/fixtures/settings_dialog.zui");
    let text = std::fs::read_to_string(source).expect("fixture source should be readable");
    let unresolved_style = "missing.review.stylesheet.asset";
    let injected = text.replace("styles = []", &format!("styles = [\"{unresolved_style}\"]"));
    assert_ne!(
        injected, text,
        "fixture should contain the empty style import list"
    );
    let target = source_root.join("settings_dialog.zui");
    std::fs::write(&target, injected).expect("temporary source should be written");

    let (_sources, unresolved_imports) =
        collect_v2_sources(std::slice::from_ref(&target)).expect("fixture should parse");
    assert_eq!(unresolved_imports.len(), 1);
    assert_eq!(unresolved_imports[0].reference, unresolved_style);
    assert_eq!(
        unresolved_imports[0].source_asset_id,
        "res://ui/runtime/fixtures/settings_dialog.zui"
    );
    std::fs::remove_dir_all(root).expect("temporary asset directory should be removed");
}

#[test]
fn optimization_batch_20260826bm_ui_v2_cache_canonical_revalidation_skips_resolve() {
    const SOURCE: &str = include_str!("../../file_cache.rs");
    let canonical_constructor = SOURCE
        .split("fn from_canonical_path(path")
        .nth(1)
        .and_then(|tail| tail.split("\n    }").next())
        .expect("canonical source-key constructor");

    assert_eq!(SOURCE_COUNT, 512);
    assert_eq!(
        SOURCE
            .matches("from_canonical_paths(&entry.source_paths)")
            .count(),
        1
    );
    assert_eq!(
        SOURCE
            .matches("from_canonical_paths(&record.source_paths)")
            .count(),
        1
    );
    assert!(!canonical_constructor.contains("canonicalize()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bm_ui_v2_cache_canonical_revalidation_p95() {
    let canonical = source_path();
    let paths = vec![canonical; SOURCE_COUNT];

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || UiV2FileStoreCacheKey::from_paths(black_box(&paths)),
        || UiV2FileStoreCacheKey::from_canonical_paths(black_box(&paths)),
    );
    assert_eq!(
        UiV2FileStoreCacheKey::from_paths(&paths),
        UiV2FileStoreCacheKey::from_canonical_paths(&paths)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME74_UI_V2_CACHE_CANONICAL_REVALIDATION_BENCH_V1 sources={SOURCE_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_redundant_canonicalize_calls={SOURCE_COUNT} optimized_redundant_canonicalize_calls=0 metadata_freshness_checks={SOURCE_COUNT} deterministic_redundant_canonicalize_reduction_percent=100.0000 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be at least 20% below legacy P95 {legacy_p95}ns"
    );
}

fn source_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/ui/v2/file_cache.rs")
        .canonicalize()
        .expect("UI v2 file cache source path")
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> UiV2FileStoreCacheKey,
    mut optimized: impl FnMut() -> UiV2FileStoreCacheKey,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for sample_index in 0..N {
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

fn benchmark_sample(operation: &mut impl FnMut() -> UiV2FileStoreCacheKey) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
