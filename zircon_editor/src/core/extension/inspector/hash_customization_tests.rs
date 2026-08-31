use std::collections::BTreeSet;
use std::hint::black_box;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;

const CUSTOMIZATION_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_CUSTOMIZATION_COUNT: usize = 8_192;
const SAMPLE_COUNT: usize = 17;

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn customization_ids() -> Vec<String> {
    (0..CUSTOMIZATION_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "plugin.generated.inspector.customization.with.long.identity.{:05}",
                (index * 4_099) % UNIQUE_CUSTOMIZATION_COUNT
            )
        })
        .collect()
}

fn ordered_unique_count(ids: &[String]) -> usize {
    let mut unique = BTreeSet::new();
    ids.iter().filter(|id| unique.insert(id.as_str())).count()
}

fn hash_unique_count(ids: &[String]) -> usize {
    let mut unique = HashSet::new();
    ids.iter().filter(|id| unique.insert(id.as_str())).count()
}

fn descriptor(id: &str) -> InspectorCustomizationDescriptor {
    InspectorCustomizationDescriptor::new(
        "fixture::target",
        "plugins://fixture/editor/inspector.zui",
        "fixture.InspectorController",
    )
    .with_id(id)
}

#[test]
fn optimization_batch_20260826ae_editor05_hash_customization_admission_preserves_order_and_duplicate_error(
) {
    let mut chain = InspectorCustomizationChain::default();
    chain
        .register(Arc::new(descriptor("fixture.first")))
        .unwrap();
    chain
        .register(Arc::new(descriptor("fixture.later")))
        .unwrap();
    let duplicate = chain.register(Arc::new(descriptor("fixture.first")));
    let target = InspectTarget::new(
        InspectTargetType::new("fixture::target").unwrap(),
        "entity:17",
    )
    .unwrap();

    assert!(matches!(
        duplicate,
        Err(InspectorRegistrationError::DuplicateCustomization(id)) if id == "fixture.first"
    ));
    assert_eq!(chain.ids.len(), 2);
    assert_eq!(chain.customizations.len(), 2);
    assert_eq!(chain.matching(&target).unwrap().id(), "fixture.first");
}

#[test]
fn optimization_batch_20260826ae_editor05_inspector_chain_uses_hash_admission_and_vector_order() {
    let source = include_str!("../inspector.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert!(production.contains("customizations: Vec<Arc<dyn InspectorCustomization>>"));
    assert!(production.contains("ids: HashSet<String>"));
    assert!(production.contains("self.customizations.iter().find"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ae_editor05_inspector_customization_hash_admission_performance_evidence(
) {
    let ids = customization_ids();
    assert_eq!(ordered_unique_count(&ids), hash_unique_count(&ids));

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&ids)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(hash_unique_count(black_box(&ids)));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_unique_count(black_box(&ids)));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_unique_count(black_box(&ids)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "EDITOR05_INSPECTOR_CUSTOMIZATION_HASH_ADMISSION_BENCH_V1 \
         admissions={CUSTOMIZATION_ADMISSION_COUNT} unique_customizations={UNIQUE_CUSTOMIZATION_COUNT} \
         vector_match_order=true ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-admission P95 {:?} exceeded 60% of ordered-admission P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
