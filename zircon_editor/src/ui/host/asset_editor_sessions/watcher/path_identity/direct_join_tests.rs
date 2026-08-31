use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::asset_id_for_watched_path;

const SAMPLE_PAIRS: usize = 21;
const OPERATIONS_PER_SAMPLE: usize = 2_048;

#[test]
fn optimization_batch_20260826hb_editor194_preserves_watched_asset_identity_contract() {
    let root = PathBuf::from("workspace").join("assets");
    let path = root.join("ui").join("menus").join("main.zui");
    assert_eq!(
        asset_id_for_watched_path(std::slice::from_ref(&root), &path).as_deref(),
        Some("res://ui/menus/main.zui")
    );
    assert_eq!(
        asset_id_for_watched_path(std::slice::from_ref(&root), &root.join("ui/main.toml")),
        None
    );
    assert_eq!(
        asset_id_for_watched_path(&[root.clone(), root], &path),
        None
    );
}

#[test]
fn optimization_batch_20260826hb_editor194_builds_the_final_asset_id_directly() {
    let source = include_str!("../path_identity.rs");
    let start = source
        .find("fn asset_id_for_watched_path(")
        .expect("asset_id_for_watched_path function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let body = &source[start..end];

    assert!(body.contains("String::with_capacity"));
    assert!(body.contains("asset_id.push_str(\"res://\")"));
    assert!(!body.contains(".replace("));
    assert!(!body.contains("format!("));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hb_editor194_watched_asset_id_direct_join_release_benchmark() {
    let root = PathBuf::from("workspace").join("assets");
    let mut path = root.clone();
    for index in 0..96 {
        path.push(format!("package-{index:03}-unicode"));
    }
    path.push("main.zui");
    let roots = [root];
    assert_eq!(
        asset_id_for_watched_path(&roots, &path),
        legacy_asset_id_for_watched_path(&roots, &path)
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_asset_id_for_watched_path(
                    black_box(&roots),
                    black_box(&path),
                ));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(asset_id_for_watched_path(
                    black_box(&roots),
                    black_box(&path),
                ));
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
        "EDITOR194_WATCHED_ASSET_ID_DIRECT_JOIN_BENCH_V1 relative_bytes={} \
         operations_per_sample={OPERATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
         legacy_ns={} optimized_ns={}",
        path.strip_prefix(&roots[0]).unwrap().as_os_str().len(),
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_asset_id_for_watched_path(asset_roots: &[PathBuf], path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_string_lossy();
    if !file_name.ends_with(".zui") {
        return None;
    }
    let mut matching_roots = asset_roots.iter().filter(|root| path.starts_with(root));
    let asset_root = matching_roots.next()?;
    if matching_roots.next().is_some() {
        return None;
    }
    let relative = path.strip_prefix(asset_root).ok()?;
    let normalized = relative.to_string_lossy().replace('\\', "/");
    Some(format!("res://{normalized}"))
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
