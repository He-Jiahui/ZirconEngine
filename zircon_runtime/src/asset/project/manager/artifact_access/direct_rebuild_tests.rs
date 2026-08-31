use std::hint::black_box;
use std::time::Instant;

use super::*;

const URI_PATH_BYTES: usize = 32 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hf_runtime252_preserves_labeled_uri_split() {
    for value in [
        "res://models/hero.glb#body",
        "lib://artifacts/hero.mesh#lod0",
        "package://starter/models/hero.glb#body",
        "builtin://meshes/cube#surface",
        "mem://preview/mesh#selected",
    ] {
        let labeled = AssetUri::parse(value).expect("labeled asset URI");
        let (source, label) = split_labeled_uri(&labeled).expect("split labeled URI");
        assert_eq!(source.scheme(), labeled.scheme());
        assert_eq!(source.path(), labeled.path());
        assert_eq!(source.label(), None);
        assert_eq!(Some(label.as_str()), labeled.label());
    }

    let unlabeled = AssetUri::parse("res://models/hero.glb").expect("unlabeled asset URI");
    assert!(split_labeled_uri(&unlabeled).is_none());
}

#[test]
fn optimization_batch_20260826hf_runtime252_rebuilds_source_from_borrowed_parts() {
    let source = include_str!("../artifact_access.rs");
    let start = source
        .find("fn split_labeled_uri(")
        .expect("split_labeled_uri function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let body = &source[start..end];

    assert!(body.contains("AssetUri::new(uri.scheme(), uri.path(), None)"));
    assert!(!body.contains("uri.to_string()"));
    assert!(!body.contains("split_once('#')"));
    assert!(!body.contains("AssetUri::parse("));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hf_runtime252_labeled_uri_direct_rebuild_release_benchmark() {
    let uri = benchmark_uri();
    assert_eq!(split_labeled_uri(&uri), legacy_split_labeled_uri(&uri));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_split_labeled_uri(black_box(&uri)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(split_labeled_uri(black_box(&uri)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME252_LABELED_URI_DIRECT_REBUILD_BENCH_V1 \
         uri_path_bytes={URI_PATH_BYTES} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_uri() -> AssetUri {
    let path = format!("models/{}", "a".repeat(URI_PATH_BYTES));
    AssetUri::parse(&format!("res://{path}#primary-mesh")).expect("benchmark URI")
}

fn legacy_split_labeled_uri(uri: &AssetUri) -> Option<(AssetUri, String)> {
    let label = uri.label()?.to_string();
    let source_text = uri.to_string().split_once('#')?.0.to_string();
    AssetUri::parse(&source_text)
        .ok()
        .map(|source_uri| (source_uri, label))
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
