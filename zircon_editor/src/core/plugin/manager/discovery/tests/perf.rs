use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::plugin::EditorPluginDescriptor;

fn catalog() -> EditorPluginCatalog {
    EditorPluginCatalog::from_descriptors(
        [
            EditorPluginDescriptor::new("plugin.alpha", "Alpha", "builtin"),
            EditorPluginDescriptor::new("plugin.beta", "Beta", "builtin"),
        ],
        std::iter::empty(),
    )
}

fn owned_ordered_membership(ids: &[String]) -> usize {
    let package_ids = ids.iter().cloned().collect::<BTreeSet<_>>();
    ids.iter()
        .rev()
        .filter(|id| package_ids.contains(id.as_str()))
        .count()
}

fn borrowed_hash_membership(ids: &[String]) -> usize {
    let mut package_ids = HashSet::with_capacity(ids.len());
    package_ids.extend(ids.iter().map(String::as_str));
    ids.iter()
        .rev()
        .filter(|id| package_ids.contains(id.as_str()))
        .count()
}

#[test]
fn optimization_batch_ia_editor610_preserves_discovery_order_and_error_precedence() {
    let catalog = catalog();
    let index = discovery_index(
        &catalog,
        [
            EditorPluginDiscovery::project("plugin.beta"),
            EditorPluginDiscovery::builtin("plugin.alpha"),
        ],
    )
    .unwrap();
    assert_eq!(
        index.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["plugin.alpha", "plugin.beta"]
    );

    assert!(matches!(
        discovery_index(
            &catalog,
            [
                EditorPluginDiscovery::builtin("plugin.missing"),
                EditorPluginDiscovery::builtin("plugin.alpha"),
            ],
        ),
        Err(EditorPluginDiscoveryError::UnknownPackage { package_id })
            if package_id == "plugin.missing"
    ));
    assert!(matches!(
        discovery_index(
            &catalog,
            [
                EditorPluginDiscovery::builtin("plugin.alpha"),
                EditorPluginDiscovery::project("plugin.alpha"),
            ],
        ),
        Err(EditorPluginDiscoveryError::DuplicateDiscovery { package_id })
            if package_id == "plugin.alpha"
    ));
}

#[test]
fn optimization_batch_ia_editor610_discovery_uses_borrowed_hash_membership() {
    let source = include_str!("../../discovery.rs");
    let function = source
        .split("pub(super) fn discovery_index")
        .nth(1)
        .expect("discovery index")
        .split("#[cfg(test)]")
        .next()
        .expect("bounded discovery index");

    assert!(source.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(function.contains("HashSet::with_capacity(package_manifests.len())"));
    assert!(function.contains("package.id.as_str()"));
    assert!(!function.contains("package.id.clone()"));
    assert!(!function.contains("collect::<BTreeSet<_>>()"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_ia_editor610_borrowed_discovery_membership_performance_evidence() {
    const PACKAGES: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let suffix = "x".repeat(224);
    let ids = (0..PACKAGES)
        .map(|index| format!("plugin.performance.{suffix}.{index:05}"))
        .collect::<Vec<_>>();
    let measure_owned = || {
        let started = Instant::now();
        black_box(owned_ordered_membership(black_box(&ids)));
        started.elapsed().as_nanos().max(1)
    };
    let measure_borrowed = || {
        let started = Instant::now();
        black_box(borrowed_hash_membership(black_box(&ids)));
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_owned());
        black_box(measure_borrowed());
    }

    let mut owned_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            owned_samples.push(measure_owned());
            borrowed_samples.push(measure_borrowed());
        } else {
            borrowed_samples.push(measure_borrowed());
            owned_samples.push(measure_owned());
        }
    }
    owned_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let owned_p50 = owned_samples[8];
    let owned_p95 = owned_samples[16];
    let borrowed_p50 = borrowed_samples[8];
    let borrowed_p95 = borrowed_samples[16];
    println!(
        "EDITOR610_BORROWED_DISCOVERY_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_owned_even owned_first_pairs=9 borrowed_first_pairs=8 packages={PACKAGES} package_id_bytes=249 owned_p50_ns={owned_p50} owned_p95_ns={owned_p95} borrowed_p50_ns={borrowed_p50} borrowed_p95_ns={borrowed_p95} temporary_package_id_clones=16384->0 membership_complexity=log_n->amortized_constant target_ratio_bp=2500"
    );
    assert!(
        borrowed_p95.saturating_mul(10_000) <= owned_p95.saturating_mul(2_500),
        "borrowed membership P95 {borrowed_p95} ns exceeded 25% of owned ordered membership {owned_p95} ns"
    );
}
