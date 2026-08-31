use std::hint::black_box;
use std::time::Instant;

use super::{ViewDescriptor, ViewRegistry};
use crate::ui::workbench::view::{ViewDescriptorId, ViewKind};

const SAMPLE_PAIRS: usize = 31;
const DESCRIPTOR_COUNT: usize = 4_096;

#[test]
fn optimization_batch_20260829au_editor266_view_registration_keeps_unique_descriptor() {
    let mut registry = ViewRegistry::default();
    let descriptor = descriptor(7);

    registry
        .register_view(descriptor.clone())
        .expect("unique descriptor should register");

    assert_eq!(registry.descriptors.len(), 1);
    assert_eq!(
        registry
            .descriptors
            .get(&descriptor.descriptor_id)
            .map(|registered| registered.default_title.as_str()),
        Some(descriptor.default_title.as_str())
    );
}

#[test]
fn optimization_batch_20260829au_editor266_view_registration_rejects_duplicate_without_replacing() {
    let mut registry = ViewRegistry::default();
    let descriptor = descriptor(7);
    registry
        .register_view(descriptor.clone())
        .expect("first descriptor should register");

    let error = registry
        .register_view(ViewDescriptor::new(
            descriptor.descriptor_id.clone(),
            ViewKind::ActivityWindow,
            "Replacement",
        ))
        .expect_err("duplicate descriptor should be rejected");

    assert!(error.contains("already registered"));
    assert_eq!(registry.descriptors.len(), 1);
    assert_eq!(
        registry
            .descriptors
            .get(&descriptor.descriptor_id)
            .map(|registered| registered.default_title.as_str()),
        Some(descriptor.default_title.as_str())
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829au_editor266_single_lookup_view_registration_bench() {
    let descriptors = (0..DESCRIPTOR_COUNT).map(descriptor).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(descriptors.clone(), false));
            optimized_samples.push(measure(descriptors.clone(), true));
        } else {
            optimized_samples.push(measure(descriptors.clone(), true));
            legacy_samples.push(measure(descriptors.clone(), false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR266_SINGLE_LOOKUP_VIEW_REGISTRATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
descriptors={DESCRIPTOR_COUNT} descriptor_id_bytes=96 legacy_hash_lookups_per_insert=2 \
optimized_hash_lookups_per_insert=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(descriptors: Vec<ViewDescriptor>, optimized: bool) -> u128 {
    let mut registry = ViewRegistry::default();
    let started = Instant::now();
    for descriptor in descriptors {
        if optimized {
            registry
                .register_view(black_box(descriptor))
                .expect("benchmark descriptors are unique");
        } else {
            legacy_register_view(&mut registry, black_box(descriptor))
                .expect("benchmark descriptors are unique");
        }
    }
    black_box(registry.descriptors.len());
    started.elapsed().as_nanos().max(1)
}

fn legacy_register_view(
    registry: &mut ViewRegistry,
    descriptor: ViewDescriptor,
) -> Result<(), String> {
    if registry.descriptors.contains_key(&descriptor.descriptor_id) {
        return Err(format!(
            "view descriptor {} already registered",
            descriptor.descriptor_id.0
        ));
    }
    registry
        .descriptors
        .insert(descriptor.descriptor_id.clone(), descriptor);
    Ok(())
}

fn descriptor(index: usize) -> ViewDescriptor {
    let suffix = format!("{index:08}");
    let fill = "x".repeat(96usize.saturating_sub("plugin..".len() + suffix.len()));
    let id = format!("plugin.{fill}.{suffix}");
    assert_eq!(id.len(), 96);
    ViewDescriptor::new(
        ViewDescriptorId::new(id),
        ViewKind::ActivityView,
        "Plugin View",
    )
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
