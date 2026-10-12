use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const ENTRY_ID_COUNT: usize = 32_768;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn entry_ids() -> Vec<String> {
    (0..ENTRY_ID_COUNT)
        .map(|index| format!("asset.generated.batch.claim.entry.{index:05}"))
        .collect()
}

fn ordered_unique_count(entry_ids: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    entry_ids
        .iter()
        .filter(|entry_id| unique.insert(entry_id.as_str()))
        .count()
}

fn hash_unique_count(entry_ids: &[String]) -> usize {
    let mut unique = HashSet::with_capacity(entry_ids.len());
    entry_ids
        .iter()
        .filter(|entry_id| unique.insert(entry_id.as_str()))
        .count()
}

#[test]
fn optimization_batch_ij_editor620_collection_claims_preserve_first_owner_conflict() {
    let asset_type = AssetTypeId::parse("sample.asset").unwrap();
    let base_owners = BTreeMap::from([("sample.existing".to_string(), "plugin.first".to_string())]);
    let pending_owners = BTreeMap::new();

    let error = validate_collection_claims(
        &asset_type,
        "creation_templates",
        "plugin.second",
        ["sample.new", "sample.existing", "sample.new"],
        &base_owners,
        &pending_owners,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        AssetTypeRegistryError::DuplicateEntryOwner {
            collection: "creation_templates",
            entry_id,
            first_owner,
            second_owner,
            ..
        } if entry_id == "sample.existing"
            && first_owner == "plugin.first"
            && second_owner == "plugin.second"
    ));
}

#[test]
fn optimization_batch_ij_editor620_collection_claims_use_size_hint_hash_capacity() {
    let source = include_str!("../batch.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(production.contains("let entry_ids = entry_ids.into_iter();"));
    assert!(production.contains("let (lower_bound, _) = entry_ids.size_hint();"));
    assert!(production.contains("let mut contribution_ids = HashSet::with_capacity(lower_bound);"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_ij_editor620_hash_batch_collection_claims_performance_evidence() {
    let entry_ids = entry_ids();
    assert_eq!(
        ordered_unique_count(&entry_ids),
        hash_unique_count(&entry_ids)
    );

    black_box(ordered_unique_count(black_box(&entry_ids)));
    black_box(hash_unique_count(black_box(&entry_ids)));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&entry_ids)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&entry_ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&entry_ids)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&entry_ids)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR620_HASH_BATCH_COLLECTION_CLAIMS_BENCH_V1 \
         entries={ENTRY_ID_COUNT} borrowed_identity=true \
         ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 40,
        "hash collection-claims P95 {:?} exceeded 40% of ordered P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
