use std::hint::black_box;
use std::time::Instant;

use crate::core::resource::{ResourceId, ResourceLocator, ResourceScheme};

use super::{resource_identity_order, ProjectResourceIdentity};

const IDENTITY_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_jf_runtime645_orders_resource_ids_without_string_formatting() {
    let source = include_str!("../../reconcile_project_resources.rs");
    let ordering = source
        .split("fn resource_identity_order")
        .nth(1)
        .expect("resource identity ordering remains present")
        .split("#[cfg(test)]")
        .next()
        .expect("resource identity ordering remains bounded");

    assert!(ordering.contains("left.id.cmp(&right.id)"));
    assert!(!ordering.contains("left.id.to_string()"));
    assert!(!ordering.contains("right.id.to_string()"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_jf_runtime645_resource_identity_order_benchmark() {
    let identities = fixture_identities();
    assert_eq!(legacy_sort(&identities), optimized_sort(&identities));

    for _ in 0..4 {
        black_box(measure_sort(&identities, false));
        black_box(measure_sort(&identities, true));
    }

    let mut string_sort_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut id_sort_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            string_sort_samples.push(measure_sort(&identities, false));
            id_sort_samples.push(measure_sort(&identities, true));
        } else {
            id_sort_samples.push(measure_sort(&identities, true));
            string_sort_samples.push(measure_sort(&identities, false));
        }
    }

    let string_sort_p95 = percentile(&string_sort_samples, 95);
    let id_sort_p95 = percentile(&id_sort_samples, 95);
    let improvement_percent = string_sort_p95
        .saturating_sub(id_sort_p95)
        .saturating_mul(100)
        / string_sort_p95.max(1);
    println!(
        "RUNTIME645_RESOURCE_IDENTITY_ORDER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} identity_count={IDENTITY_COUNT} string_sort_ns={} id_sort_ns={} string_sort_p95_ns={string_sort_p95} id_sort_p95_ns={id_sort_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&string_sort_samples),
        csv(&id_sort_samples),
    );
    assert!(id_sort_p95 <= string_sort_p95 * 80 / 100);
}

fn fixture_identities() -> Vec<ProjectResourceIdentity> {
    let locator = ResourceLocator::new(ResourceScheme::Res, "runtime645/shared.zasset", None)
        .expect("synthetic locator");
    (0..IDENTITY_COUNT)
        .rev()
        .map(|index| ProjectResourceIdentity {
            id: ResourceId::from_stable_label(&format!("runtime645-{index:08}")),
            locator: locator.clone(),
        })
        .collect()
}

fn measure_sort(identities: &[ProjectResourceIdentity], optimized: bool) -> u128 {
    let started = Instant::now();
    let sorted = if optimized {
        optimized_sort(identities)
    } else {
        legacy_sort(identities)
    };
    black_box(sorted);
    started.elapsed().as_nanos().max(1)
}

fn legacy_sort(identities: &[ProjectResourceIdentity]) -> Vec<ResourceId> {
    let mut sorted = identities.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        left.locator
            .cmp(&right.locator)
            .then_with(|| left.id.to_string().cmp(&right.id.to_string()))
    });
    sorted.into_iter().map(|identity| identity.id).collect()
}

fn optimized_sort(identities: &[ProjectResourceIdentity]) -> Vec<ResourceId> {
    let mut sorted = identities.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| resource_identity_order(left, right));
    sorted.into_iter().map(|identity| identity.id).collect()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
