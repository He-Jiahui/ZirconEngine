use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::plugin::PluginPackageManifest;

use super::*;

fn ordered_membership(ids: &[String]) -> usize {
    let package_ids = ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    ids.iter()
        .rev()
        .filter(|id| package_ids.contains(id.as_str()))
        .count()
}

fn hash_membership(ids: &[String]) -> usize {
    let mut package_ids = HashSet::with_capacity(ids.len());
    package_ids.extend(ids.iter().map(String::as_str));
    ids.iter()
        .rev()
        .filter(|id| package_ids.contains(id.as_str()))
        .count()
}

#[test]
fn optimization_batch_ib_editor611_preserves_editor_scoped_duplicate_manifest_diagnostics() {
    let catalog = EditorPluginCatalog::from_descriptors(
        [EditorPluginDescriptor::new(
            "plugin.alpha",
            "Alpha",
            "builtin",
        )],
        [
            PluginPackageManifest::new("plugin.alpha", "Alpha First"),
            PluginPackageManifest::new("plugin.alpha", "Alpha Duplicate"),
            PluginPackageManifest::new("runtime.only", "Runtime First"),
            PluginPackageManifest::new("runtime.only", "Runtime Duplicate"),
        ],
    );

    assert_eq!(
        catalog.admission_duplicate_package_ids,
        BTreeSet::from(["plugin.alpha".to_string()])
    );
    assert_eq!(catalog.registrations.len(), 1);
    assert_eq!(catalog.registrations[0].package_manifest.id, "plugin.alpha");
}

#[test]
fn optimization_batch_ib_editor611_descriptor_ids_use_preallocated_hash_membership() {
    let source = include_str!("../../catalog.rs");
    let constructor = source
        .split("pub(crate) fn from_descriptors")
        .nth(1)
        .expect("descriptor catalog constructor")
        .split("pub(crate) fn builtin")
        .next()
        .expect("bounded descriptor catalog constructor");

    assert!(source.contains("use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};"));
    assert!(constructor.contains("HashSet::with_capacity(descriptors.len())"));
    assert!(constructor.contains("descriptor.package_id.as_str()"));
    assert!(!constructor.contains("collect::<BTreeSet<_>>()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_ib_editor611_hash_descriptor_membership_performance_evidence() {
    const DESCRIPTORS: usize = 32_768;
    const SAMPLE_PAIRS: usize = 17;
    let suffix = "x".repeat(160);
    let ids = (0..DESCRIPTORS)
        .map(|index| format!("plugin.descriptor.{suffix}.{index:05}"))
        .collect::<Vec<_>>();
    let measure_ordered = || {
        let started = Instant::now();
        black_box(ordered_membership(black_box(&ids)));
        started.elapsed().as_nanos().max(1)
    };
    let measure_hash = || {
        let started = Instant::now();
        black_box(hash_membership(black_box(&ids)));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_ordered());
        black_box(measure_hash());
    }

    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            ordered_samples.push(measure_ordered());
            hash_samples.push(measure_hash());
        } else {
            hash_samples.push(measure_hash());
            ordered_samples.push(measure_ordered());
        }
    }
    ordered_samples.sort_unstable();
    hash_samples.sort_unstable();
    let ordered_p50 = ordered_samples[8];
    let ordered_p95 = ordered_samples[16];
    let hash_p50 = hash_samples[8];
    let hash_p95 = hash_samples[16];
    println!(
        "EDITOR611_HASH_DESCRIPTOR_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_ordered_even ordered_first_pairs=9 hash_first_pairs=8 descriptors={DESCRIPTORS} package_id_bytes=184 ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} membership_complexity=log_n->amortized_constant target_ratio_bp=4000"
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= ordered_p95.saturating_mul(4_000),
        "hash descriptor membership P95 {hash_p95} ns exceeded 40% of ordered membership {ordered_p95} ns"
    );
}
