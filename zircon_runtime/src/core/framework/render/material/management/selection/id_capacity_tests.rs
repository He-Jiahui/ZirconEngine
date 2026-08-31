use std::hint::black_box;
use std::time::Instant;

use super::{
    unique_material_ids, RenderMaterialManagementRecord, RenderMaterialManagementSelection,
    ResourceId,
};
use crate::core::framework::render::material::management::RenderMaterialManagementSnapshot;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const IDS_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826eu_runtime190_capacity_preserves_unique_selection_order() {
    let material_ids = material_ids();
    let records = material_ids[..128]
        .iter()
        .enumerate()
        .map(|(index, material_id)| RenderMaterialManagementRecord {
            material_id: *material_id,
            material_name: Some(format!("Material {index}")),
            snapshot: RenderMaterialManagementSnapshot::default(),
        })
        .collect::<Vec<_>>();
    let requests = material_ids
        .iter()
        .copied()
        .chain(material_ids.iter().copied())
        .collect::<Vec<_>>();

    let selection = RenderMaterialManagementSelection::from_records(&records, requests);
    let unique = unique_material_ids(material_ids.iter().copied());

    assert_eq!(selection.requested_count, IDS_PER_BUILD);
    assert_eq!(selection.records.len(), 128);
    assert_eq!(selection.missing_material_ids.len(), 128);
    assert_eq!(selection.records[0].material_id, material_ids[0]);
    assert_eq!(selection.records[127].material_id, material_ids[127]);
    assert_eq!(selection.missing_material_ids[0], material_ids[128]);
    assert_eq!(unique, material_ids);
    assert!(unique.capacity() >= IDS_PER_BUILD);
}

#[test]
fn optimization_batch_20260826eu_runtime190_unique_ids_reserve_iterator_lower_bound() {
    let source = include_str!("../selection.rs");
    assert!(source.contains("let (minimum_ids, _) = material_ids.size_hint();"));
    assert!(source.contains("Vec::with_capacity(minimum_ids)"));
    assert!(source.contains("HashSet::with_capacity(minimum_ids)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eu_runtime190_material_selection_id_capacity_bench() {
    let ids = material_ids();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&ids, false));
            optimized_samples.push(measure(&ids, true));
        } else {
            optimized_samples.push(measure(&ids, true));
            legacy_samples.push(measure(&ids, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME190_MATERIAL_SELECTION_ID_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} unique_ids_per_build={IDS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn material_ids() -> Vec<ResourceId> {
    (0..IDS_PER_BUILD)
        .map(|index| ResourceId::from_stable_label(&format!("material:runtime190:{index}")))
        .collect()
}

fn measure(ids: &[ResourceId], reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut unique = if reserve {
            Vec::with_capacity(IDS_PER_BUILD)
        } else {
            Vec::new()
        };
        let mut seen = if reserve {
            std::collections::HashSet::with_capacity(IDS_PER_BUILD)
        } else {
            std::collections::HashSet::new()
        };
        for material_id in ids {
            if seen.insert(black_box(*material_id)) {
                unique.push(*material_id);
            }
        }
        checksum ^= black_box(unique.len() ^ unique.capacity() ^ seen.len() ^ seen.capacity());
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
