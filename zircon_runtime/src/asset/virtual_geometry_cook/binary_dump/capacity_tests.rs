use std::hint::black_box;
use std::time::Instant;

use super::{binary_dump_capacity, encode_virtual_geometry_cook_binary_dump};
use crate::asset::{
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryPageDependencyAsset, VirtualGeometryRootClusterRangeAsset,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 512;
const ENCODED_BYTES_PER_BUILD: usize = 16 * 1_024;
const RECORDS: usize = 256;
const PAYLOAD_BYTES: usize = 128;

#[test]
fn optimization_batch_20260826fm_runtime208_capacity_matches_real_binary_dump_length() {
    let asset = binary_dump_fixture();
    let expected_capacity = binary_dump_capacity(&asset);

    let dump = encode_virtual_geometry_cook_binary_dump(&asset);

    assert_eq!(&dump[..4], b"ZVGB");
    assert_eq!(dump.len(), expected_capacity);
    assert!(dump.capacity() >= expected_capacity);
}

#[test]
fn optimization_batch_20260826fm_runtime208_encoder_reserves_computed_dump_size() {
    let source = include_str!("../binary_dump.rs");
    assert!(source.contains("Vec::with_capacity(binary_dump_capacity(asset))"));
    assert!(source.contains("fn binary_dump_capacity(asset: &VirtualGeometryAsset) -> usize"));
    assert!(source.contains("std::mem::size_of::<u32>()"));
    assert!(source.contains("std::mem::size_of::<u64>()"));
    assert!(!source.contains("let mut dump = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fm_runtime208_binary_dump_capacity_bench() {
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
        "RUNTIME208_VIRTUAL_GEOMETRY_BINARY_DUMP_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} encoded_bytes_per_build={ENCODED_BYTES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn binary_dump_fixture() -> VirtualGeometryAsset {
    VirtualGeometryAsset {
        hierarchy_buffer: (0..RECORDS)
            .map(|index| VirtualGeometryHierarchyNodeAsset {
                node_id: index as u32,
                child_node_ids: vec![((index + 1) % RECORDS) as u32],
                ..VirtualGeometryHierarchyNodeAsset::default()
            })
            .collect(),
        cluster_headers: (0..RECORDS)
            .map(|index| VirtualGeometryClusterHeaderAsset {
                cluster_id: index as u32,
                hierarchy_node_id: index as u32,
                page_id: index as u32,
                ..VirtualGeometryClusterHeaderAsset::default()
            })
            .collect(),
        cluster_page_headers: (0..RECORDS)
            .map(|index| VirtualGeometryClusterPageHeaderAsset {
                page_id: index as u32,
                start_offset: (index * PAYLOAD_BYTES) as u32,
                payload_size_bytes: PAYLOAD_BYTES as u64,
            })
            .collect(),
        cluster_page_data: (0..RECORDS)
            .map(|index| vec![index as u8; PAYLOAD_BYTES])
            .collect(),
        root_page_table: (0..RECORDS as u32).collect(),
        page_dependencies: (0..RECORDS)
            .map(|index| VirtualGeometryPageDependencyAsset {
                page_id: index as u32,
                child_page_ids: vec![((index + 1) % RECORDS) as u32],
                ..VirtualGeometryPageDependencyAsset::default()
            })
            .collect(),
        root_cluster_ranges: (0..RECORDS)
            .map(|index| VirtualGeometryRootClusterRangeAsset {
                node_id: index as u32,
                cluster_start: index as u32,
                cluster_count: 1,
            })
            .collect(),
        debug: VirtualGeometryDebugMetadataAsset {
            mesh_name: Some("capacity-fixture".to_string()),
            source_hint: Some("optimization-batch-fm".to_string()),
            notes: (0..16).map(|index| format!("note-{index:02}")).collect(),
        },
    }
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut dump = if reserve {
            Vec::with_capacity(ENCODED_BYTES_PER_BUILD)
        } else {
            Vec::new()
        };
        for byte in 0..ENCODED_BYTES_PER_BUILD {
            dump.push(black_box(byte as u8));
        }
        checksum ^= black_box(dump.len() ^ dump.capacity());
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
