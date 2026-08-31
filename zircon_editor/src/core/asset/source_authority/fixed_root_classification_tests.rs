use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::resource::ResourceLocator;

use super::{AssetSourceAuthority, AssetSourceKind, AssetSourceWritePolicy};

const SAMPLE_PAIRS: usize = 31;
const CLASSIFICATIONS_PER_SAMPLE: usize = 100_000;
const FIXED_ROOTS: [&str; 4] = ["res://", "lib://", "builtin://", "mem://"];

#[test]
fn optimization_batch_20260828ir_editor236_fixed_roots_preserve_authority_semantics() {
    let expected = [
        AssetSourceKind::Project,
        AssetSourceKind::Library,
        AssetSourceKind::Builtin,
        AssetSourceKind::Transient,
    ];

    for (target, expected_kind) in FIXED_ROOTS.into_iter().zip(expected) {
        let authority =
            AssetSourceAuthority::from_target_str(AssetSourceWritePolicy::ProjectOnly, target)
                .expect("fixed root authority");
        assert_eq!(authority.kind(), expected_kind);
        assert_eq!(
            authority,
            legacy_fixed_root_authority(AssetSourceWritePolicy::ProjectOnly, target)
        );
    }
}

#[test]
fn optimization_batch_20260828ir_editor236_direct_match_precedes_locator_fallback() {
    let source = include_str!("../source_authority.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let from_target = implementation
        .split("pub fn from_target_str")
        .nth(1)
        .expect("target authority implementation");
    let direct_match = from_target
        .find("let fixed_root_kind = match target")
        .expect("fixed-root match");
    let locator_fallback = from_target
        .find("ResourceLocator::parse")
        .expect("locator fallback");

    assert!(direct_match < locator_fallback);
    assert!(from_target.contains("if let Some(kind) = fixed_root_kind"));
    assert!(from_target.contains("return Ok(Self::new(policy, kind));"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260828ir_editor236_direct_fixed_root_authority_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR236_DIRECT_FIXED_ROOT_AUTHORITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
classifications_per_sample={CLASSIFICATIONS_PER_SAMPLE} fixed_root_count={} \
legacy_locator_parses_per_classification=1 optimized_locator_parses_per_classification=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        FIXED_ROOTS.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_fixed_root_authority(
    policy: AssetSourceWritePolicy,
    target: &str,
) -> AssetSourceAuthority {
    let normalized_root = format!("{target}__root__");
    let locator = ResourceLocator::parse(normalized_root.as_str()).expect("fixed root locator");
    AssetSourceAuthority::from_locator(policy, &locator)
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for iteration in 0..CLASSIFICATIONS_PER_SAMPLE {
        let target = black_box(FIXED_ROOTS[iteration % FIXED_ROOTS.len()]);
        let authority = if optimized {
            AssetSourceAuthority::from_target_str(AssetSourceWritePolicy::ProjectOnly, target)
                .expect("optimized fixed root")
        } else {
            legacy_fixed_root_authority(AssetSourceWritePolicy::ProjectOnly, target)
        };
        checksum ^= black_box((authority.kind() as usize).wrapping_add(iteration));
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
