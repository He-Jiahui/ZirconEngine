use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const VALIDATIONS_PER_SAMPLE: usize = 100_000;

#[test]
fn optimization_batch_20260826cl_editor_shell_region_bitset_preserves_duplicate_and_missing_errors()
{
    let asset_source = include_str!("../../../../../assets/ui/editor/layout/shell_regions.toml");
    let asset = WorkbenchShellRegionsAsset::from_toml_str(asset_source).expect("valid asset");
    assert_eq!(asset.regions.len(), EditorRegion::ALL.len());

    let mut duplicate = WorkbenchSkeleton::jetbrains_default().regions;
    let repeated = duplicate[0].clone();
    duplicate[1] = repeated;
    assert!(matches!(
        validate_complete_region_set(&duplicate),
        Err(WorkbenchShellRegionsAssetError::DuplicateRegion {
            region: EditorRegion::LeftTop,
        })
    ));

    let mut missing = WorkbenchSkeleton::jetbrains_default().regions;
    missing.retain(|binding| binding.region != EditorRegion::Center);
    assert!(matches!(
        validate_complete_region_set(&missing),
        Err(WorkbenchShellRegionsAssetError::MissingRegion {
            region: EditorRegion::Center,
        })
    ));
}

#[test]
fn optimization_batch_20260826cl_editor_shell_region_validation_uses_fixed_bitset() {
    let source = include_str!("../shell_regions_asset.rs");
    let validation = source
        .split("fn validate_complete_region_set")
        .nth(1)
        .expect("region validation implementation");

    assert!(validation.contains("let mut occupied_regions = 0u8"));
    assert!(validation.contains("editor_region_bit"));
    assert!(!validation.contains("BTreeSet"));
}

fn measure_legacy(regions: &[EditorRegion]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..VALIDATIONS_PER_SAMPLE {
        let mut seen = BTreeSet::new();
        for region in black_box(regions) {
            checksum = checksum.wrapping_add(seen.insert(*region) as usize);
        }
        for region in EditorRegion::ALL {
            checksum = checksum.wrapping_add(seen.contains(&region) as usize);
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(regions: &[EditorRegion]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..VALIDATIONS_PER_SAMPLE {
        let mut occupied = 0u8;
        for region in black_box(regions) {
            let bit = editor_region_bit(*region);
            checksum = checksum.wrapping_add((occupied & bit == 0) as usize);
            occupied |= bit;
        }
        for region in EditorRegion::ALL {
            checksum = checksum.wrapping_add((occupied & editor_region_bit(region) != 0) as usize);
        }
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

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
#[ignore = "release-only shell region validation benchmark"]
fn optimization_batch_20260826cl_editor_shell_region_bitset_release_benchmark() {
    let regions = EditorRegion::ALL;
    for _ in 0..4 {
        black_box(measure_legacy(&regions));
        black_box(measure_optimized(&regions));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&regions));
            optimized_samples.push(measure_optimized(&regions));
        } else {
            optimized_samples.push(measure_optimized(&regions));
            legacy_samples.push(measure_legacy(&regions));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR54_SHELL_REGION_BITSET_VALIDATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
validations_per_sample={VALIDATIONS_PER_SAMPLE} region_count={} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_tree_instances_per_sample={VALIDATIONS_PER_SAMPLE} \
optimized_tree_instances_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        regions.len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "region bitset must reduce P95 by at least 30%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
