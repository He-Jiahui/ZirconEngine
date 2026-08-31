use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::ui::template::{UiAssetLoader, UiDocumentCompiler};

use super::*;

const UNIQUE_ASSET_COUNT: usize = 8_192;
const ADMISSION_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn layout_source(asset_id: &str) -> String {
    format!(
        r#"
[asset]
kind = "layout"
id = "{asset_id}"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
"#
    )
}

fn legacy_owned_ordered_membership(asset_ids: &[&str], probes: &[String]) -> usize {
    let asset_ids = asset_ids
        .iter()
        .copied()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    probes
        .iter()
        .filter(|probe| asset_ids.contains(probe.as_str()))
        .count()
}

fn borrowed_hash_membership(asset_ids: &[&str], probes: &[String]) -> usize {
    let asset_ids = asset_ids.iter().copied().collect::<HashSet<_>>();
    probes
        .iter()
        .filter(|probe| asset_ids.contains(probe.as_str()))
        .count()
}

#[test]
fn optimization_batch_20260826ah_runtime74_hash_eviction_preserves_multi_asset_semantics() {
    let compiler = UiDocumentCompiler::default();
    let documents = [
        "runtime.cache.alpha",
        "runtime.cache.beta",
        "runtime.cache.gamma",
    ]
    .map(|asset_id| UiAssetLoader::load_toml_str(&layout_source(asset_id)).unwrap());
    let mut cache = UiAssetCompileCache::new();
    for document in &documents {
        assert!(
            !compiler
                .compile_with_cache(document, &mut cache)
                .unwrap()
                .cache_hit
        );
    }

    let report = cache.evict_assets([
        "runtime.cache.alpha",
        "runtime.cache.alpha",
        "runtime.cache.gamma",
        "runtime.cache.missing",
    ]);

    assert_eq!(
        report,
        UiAssetCompileCacheEvictionReport {
            entries_removed: 2,
            snapshots_removed: 2,
        }
    );
    assert_eq!(cache.len(), 1);
    assert!(
        !compiler
            .compile_with_cache(&documents[0], &mut cache)
            .unwrap()
            .cache_hit
    );
    assert!(
        compiler
            .compile_with_cache(&documents[1], &mut cache)
            .unwrap()
            .cache_hit
    );
    assert!(
        !compiler
            .compile_with_cache(&documents[2], &mut cache)
            .unwrap()
            .cache_hit
    );
}

#[test]
fn optimization_batch_20260826ah_runtime74_compile_cache_uses_borrowed_hash_eviction() {
    let source = include_str!("../compile_cache.rs");
    let eviction = source
        .split("pub fn evict_assets")
        .nth(1)
        .and_then(|body| body.split("pub fn get").next())
        .expect("compile-cache eviction implementation");

    assert!(source.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(eviction.contains("collect::<HashSet<_>>()"));
    assert!(eviction.contains("contains(compiled.asset.id.as_str())"));
    assert!(!eviction.contains("map(str::to_string)"));
    assert!(!eviction.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ah_runtime74_compile_cache_hash_eviction_performance_evidence() {
    let unique_ids = (0..UNIQUE_ASSET_COUNT)
        .map(|index| format!("runtime_cache_asset_with_long_identity_{index:05}"))
        .collect::<Vec<_>>();
    let admissions = (0..ADMISSION_COUNT)
        .map(|index| unique_ids[index % UNIQUE_ASSET_COUNT].as_str())
        .collect::<Vec<_>>();
    let probes = (0..UNIQUE_ASSET_COUNT * 2)
        .map(|index| format!("runtime_cache_asset_with_long_identity_{index:05}"))
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_owned_ordered_membership(&admissions, &probes),
        borrowed_hash_membership(&admissions, &probes)
    );

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_owned_ordered_membership(
                black_box(&admissions),
                black_box(&probes),
            ));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(borrowed_hash_membership(
                black_box(&admissions),
                black_box(&probes),
            ));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(borrowed_hash_membership(
                black_box(&admissions),
                black_box(&probes),
            ));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_owned_ordered_membership(
                black_box(&admissions),
                black_box(&probes),
            ));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "RUNTIME74_COMPILE_CACHE_HASH_EVICTION_BENCH_V1 \
         admissions={ADMISSION_COUNT} unique_assets={UNIQUE_ASSET_COUNT} \
         borrowed_asset_identity=true ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "borrowed hash-eviction P95 {:?} exceeded 60% of owned ordered-eviction P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
