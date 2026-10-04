//! Separate unfiltered listing evidence; the original readiness and G39 query cohorts stay intact.

use super::{entry_matches_filter, AssetRegistryFilter, AssetRegistryIndex};
use crate::asset::registry::AssetRegistryEntry;
use crate::asset::{AssetKind, AssetUri, AssetUuid};
use std::hint::black_box;
use std::time::Instant;

// Reuse the actual integration helper without introducing another allocator or OS service.
#[path = "../../../../../tests/resource_registry_g39_profile/process_counters.rs"]
mod process_counters;

const SCALES: [usize; 3] = [10_000, 100_000, 1_000_000];
const WARMUP_PAIRS: usize = 3;
const SAMPLE_PAIRS: usize = 101;

fn fixture(count: usize) -> AssetRegistryIndex {
    AssetRegistryIndex::from_entries((0..count).rev().map(|i| {
        AssetRegistryEntry::new(
            AssetUuid::from_stable_label(&format!("runtime51-unfiltered-listing-{i}")),
            AssetUri::parse(&format!("res://runtime51-listing/{i:07}.asset")).unwrap(),
            AssetKind::Data,
            "unfiltered-listing-fixture",
        )
    }))
    .expect("unique registry fixture")
}

// Exact original default-filter producer body: same borrowed candidate rows,
// same predicate and path comparator. This is not the older whole-table HashMap scan.
fn previous_get_assets<'a>(
    index: &'a AssetRegistryIndex,
    filter: &AssetRegistryFilter,
) -> Vec<&'a AssetRegistryEntry> {
    let mut entries = index.candidate_entries(filter);
    entries.retain(|entry| entry_matches_filter(entry, filter));
    entries.sort_by(|left, right| left.path().cmp(right.path()));
    entries
}

fn execute<'a>(
    index: &'a AssetRegistryIndex,
    filter: &AssetRegistryFilter,
    previous: bool,
) -> Vec<&'a AssetRegistryEntry> {
    if previous {
        previous_get_assets(black_box(index), black_box(filter))
    } else {
        black_box(index).get_assets(black_box(filter))
    }
}

struct Sample {
    wall_ns: u128,
    before: process_counters::Snapshot,
    after: process_counters::Snapshot,
    delta: process_counters::Delta,
}

fn measure(index: &AssetRegistryIndex, filter: &AssetRegistryFilter, previous: bool) -> Sample {
    let start = Instant::now();
    let output = execute(index, filter, previous);
    black_box(&output);
    let wall_ns = start.elapsed().as_nanos();
    assert_eq!(output.len(), index.len());
    drop(output);

    // Another equivalent query brackets real process counters; output stays live
    // through the after snapshot. CPU includes bracket overhead and all threads.
    let before = process_counters::Snapshot::current().expect("process counters before");
    let output = execute(index, filter, previous);
    black_box(&output);
    let after = process_counters::Snapshot::current().expect("process counters after");
    assert_eq!(output.len(), index.len());
    drop(output);
    Sample {
        wall_ns,
        before,
        after,
        delta: process_counters::Delta::between(before, after),
    }
}

fn pair(
    index: &AssetRegistryIndex,
    filter: &AssetRegistryFilter,
    previous_first: bool,
) -> (Sample, Sample) {
    if previous_first {
        (measure(index, filter, true), measure(index, filter, false))
    } else {
        let candidate = measure(index, filter, false);
        let previous = measure(index, filter, true);
        (previous, candidate)
    }
}

fn percentile(values: &[u128], percent: usize) -> u128 {
    let mut values = values.to_vec();
    values.sort_unstable();
    values[(values.len() * percent).div_ceil(100).saturating_sub(1)]
}

#[test]
#[ignore = "separate Windows Release unfiltered listing evidence; --nocapture --test-threads=1"]
fn runtime51_unfiltered_listing_release_profile() {
    assert!(!cfg!(debug_assertions), "requires Release");
    println!(
        "r51_listing_environment,samples={},warmups={},pair_order=alternating,baseline=exact_previous_unfiltered_query_body,output=owned_vec_borrowed_rows,output_drop=outside_wall_timer,cpu_unit=100ns_process_sum_including_bracket_overhead,rss=working_set_endpoints,peak_rss=process_lifetime_not_query_peak,io=process_operations_and_transfers_not_disk_only,allocation=unavailable_not_zero,allocation_reason=existing_Runtime_unit_allocator_private_to_native_plugin_tests,no_new_allocator=true,original22_readiness_cases=unchanged,original24_G39_dependency_cases=unchanged,numeric_budget=pending,query_source_blake3={},index_source_blake3={},profile_source_blake3={},counter_source_blake3={}",
        SAMPLE_PAIRS,
        WARMUP_PAIRS,
        blake3::hash(include_bytes!("../../query.rs")).to_hex(),
        blake3::hash(include_bytes!("../../asset_registry_index.rs")).to_hex(),
        blake3::hash(include_bytes!("unfiltered_profile.rs")).to_hex(),
        blake3::hash(include_bytes!(
            "../../../../../tests/resource_registry_g39_profile/process_counters.rs"
        ))
        .to_hex(),
    );
    for count in SCALES {
        let index = fixture(count);
        let filter = AssetRegistryFilter::default();
        let previous = previous_get_assets(&index, &filter);
        let candidate = index.get_assets(&filter);
        assert_eq!(previous, candidate);
        assert!(previous
            .iter()
            .zip(&candidate)
            .all(|(old, new)| std::ptr::eq(*old, *new)));
        drop(previous);
        drop(candidate);
        for warmup in 0..WARMUP_PAIRS {
            let _ = pair(&index, &filter, warmup % 2 == 0);
        }
        let mut raw = Vec::with_capacity(SAMPLE_PAIRS);
        for sample in 0..SAMPLE_PAIRS {
            raw.push(pair(&index, &filter, sample % 2 == 0));
        }
        let previous = raw.iter().map(|(old, _)| old.wall_ns).collect::<Vec<_>>();
        let candidate = raw.iter().map(|(_, new)| new.wall_ns).collect::<Vec<_>>();
        println!(
            "r51_listing_summary,entries={},samples={},previous_p50_ns={},previous_p95_ns={},previous_p99_ns={},candidate_p50_ns={},candidate_p95_ns={},candidate_p99_ns={},numeric_budget=pending",
            count,
            SAMPLE_PAIRS,
            percentile(&previous, 50),
            percentile(&previous, 95),
            percentile(&previous, 99),
            percentile(&candidate, 50),
            percentile(&candidate, 95),
            percentile(&candidate, 99),
        );
        for (sample, (old, new)) in raw.into_iter().enumerate() {
            println!(
                "r51_listing_raw,entries={},pair={},previous_first={},previous_ns={},candidate_ns={}",
                count,
                sample,
                sample % 2 == 0,
                old.wall_ns,
                new.wall_ns,
            );
            for (arm, values) in [("previous", old), ("candidate", new)] {
                println!(
                    "r51_listing_process_raw,entries={},pair={},arm={},before={:?},after={:?},delta={:?}",
                    count, sample, arm, values.before, values.after, values.delta,
                );
            }
        }
    }
}
