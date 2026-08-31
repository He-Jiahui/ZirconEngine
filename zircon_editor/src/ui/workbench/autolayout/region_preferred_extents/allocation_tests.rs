use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::ui::workbench::autolayout::ShellSizePx;

const LOOKUP_BATCHES: usize = 16_384;
const SAMPLE_PAIRS: usize = 21;
const REGIONS: [ShellRegionId; 4] = [
    ShellRegionId::Left,
    ShellRegionId::Document,
    ShellRegionId::Right,
    ShellRegionId::Bottom,
];

fn physical_extents() -> BTreeMap<ShellRegionId, f32> {
    BTreeMap::from([
        (ShellRegionId::Left, 480.0),
        (ShellRegionId::Document, 1_920.0),
        (ShellRegionId::Right, 600.0),
        (ShellRegionId::Bottom, 720.0),
    ])
}

fn resolution() -> ResolutionContext {
    ResolutionContext::from_physical_size(ShellSizePx::new(3_840.0, 2_160.0), 2.0)
}

#[test]
fn optimization_batch_20260826cn_editor77_region_preferred_lookup_preserves_scale_and_absence() {
    let mut physical = physical_extents();
    physical.remove(&ShellRegionId::Document);
    let preferred = LogicalRegionPreferredExtents::new(Some(&physical), resolution());

    assert_eq!(preferred.get(ShellRegionId::Left), Some(240.0));
    assert_eq!(preferred.get(ShellRegionId::Bottom), Some(360.0));
    assert_eq!(preferred.get(ShellRegionId::Document), None);
    assert_eq!(
        LogicalRegionPreferredExtents::new(None, resolution()).get(ShellRegionId::Left),
        None
    );
}

#[test]
fn optimization_batch_20260826cn_editor77_geometry_borrows_preferred_maps_without_collecting() {
    let compute_source = include_str!("../geometry/compute.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let preferred_source = include_str!("../region_preferred_extents.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(compute_source.contains("LogicalRegionPreferredExtents::new"));
    assert!(!compute_source.contains("collect::<BTreeMap"));
    assert!(preferred_source.contains("self.physical"));
    assert!(preferred_source.contains(".and_then"));
}

fn legacy_lookup_batch(physical: &BTreeMap<ShellRegionId, f32>) -> f32 {
    let resolution = resolution();
    let mut checksum = 0.0;
    for _ in 0..LOOKUP_BATCHES {
        let logical = physical
            .iter()
            .map(|(region, extent)| (*region, resolution.to_logical(*extent)))
            .collect::<BTreeMap<_, _>>();
        for region in REGIONS {
            checksum += logical.get(&region).copied().unwrap_or_default();
        }
    }
    checksum
}

fn optimized_lookup_batch(physical: &BTreeMap<ShellRegionId, f32>) -> f32 {
    let preferred = LogicalRegionPreferredExtents::new(Some(physical), resolution());
    let mut checksum = 0.0;
    for _ in 0..LOOKUP_BATCHES {
        for region in REGIONS {
            checksum += preferred.get(region).unwrap_or_default();
        }
    }
    checksum
}

fn elapsed_ns(run: impl FnOnce() -> f32) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826cn_editor77_region_preferred_lookup_performance_evidence() {
    let physical = physical_extents();
    for _ in 0..3 {
        assert_eq!(
            black_box(legacy_lookup_batch(&physical)),
            optimized_lookup_batch(&physical)
        );
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_ns(|| legacy_lookup_batch(&physical)));
            optimized_samples.push(elapsed_ns(|| optimized_lookup_batch(&physical)));
        } else {
            optimized_samples.push(elapsed_ns(|| optimized_lookup_batch(&physical)));
            legacy_samples.push(elapsed_ns(|| legacy_lookup_batch(&physical)));
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples, 95);
    println!(
        "EDITOR77_REGION_PREFERRED_ZERO_ALLOCATION_LOOKUP_BENCH_V1 sample_pairs={} lookup_batches={} regions_per_batch={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SAMPLE_PAIRS,
        LOOKUP_BATCHES,
        REGIONS.len(),
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed preferred extent lookup p95 must be at least 30% below collected map projection: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
