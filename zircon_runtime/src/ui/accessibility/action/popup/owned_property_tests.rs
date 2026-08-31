use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ie_runtime_popup_dismiss_moves_owned_property() {
    let property = "popup-property/".repeat(4 * 1024);
    let allocation = property.as_ptr();

    let request = popup_dismiss_mutation_request(UiNodeId::new(89), property);

    assert_eq!(request.property.as_ptr(), allocation);
    assert_eq!(request.value, UiValue::Bool(false));
    assert_eq!(request.source, UiReflectedPropertySource::RuntimeState);
}

#[test]
fn optimization_batch_20260828ie_runtime_popup_dismiss_consumes_lookup_property() {
    let source = include_str!("../popup.rs");
    let dispatch = source
        .split("pub(super) fn dispatch_dismiss")
        .nth(1)
        .and_then(|body| body.split("fn popup_dismiss_mutation_request").next())
        .expect("accessibility popup dismiss implementation");
    let request = source
        .split("fn popup_dismiss_mutation_request")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("popup dismiss request implementation");

    assert!(dispatch.contains("popup_dismiss_mutation_request(popup_id, property)"));
    assert!(!dispatch.contains("property.clone()"));
    assert!(request.contains("UiPropertyMutationRequest::accessibility_action("));
    assert!(request.contains("property,"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ie_runtime_owned_popup_dismiss_property_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;

    black_box(legacy_popup_dismiss_mutation_request(
        UiNodeId::new(89),
        benchmark_property(64 * 1024),
    ));
    black_box(popup_dismiss_mutation_request(
        UiNodeId::new(89),
        benchmark_property(64 * 1024),
    ));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_properties = (0..ITERATIONS)
            .map(|_| benchmark_property(64 * 1024))
            .collect::<Vec<_>>();
        let optimized_properties = (0..ITERATIONS)
            .map(|_| benchmark_property(64 * 1024))
            .collect::<Vec<_>>();
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_properties(
                legacy_properties,
                legacy_popup_dismiss_mutation_request,
            ));
            optimized_samples.push(measure_properties(
                optimized_properties,
                popup_dismiss_mutation_request,
            ));
        } else {
            optimized_samples.push(measure_properties(
                optimized_properties,
                popup_dismiss_mutation_request,
            ));
            legacy_samples.push(measure_properties(
                legacy_properties,
                legacy_popup_dismiss_mutation_request,
            ));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME277_OWNED_POPUP_DISMISS_PROPERTY_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_property(bytes: usize) -> String {
    "popup-dismiss-property/".repeat(bytes / 23)
}

fn legacy_popup_dismiss_mutation_request(
    popup_id: UiNodeId,
    property: String,
) -> UiPropertyMutationRequest {
    UiPropertyMutationRequest::accessibility_action(
        popup_id,
        property.clone(),
        UiValue::Bool(false),
    )
    .with_source(UiReflectedPropertySource::RuntimeState)
}

fn measure_properties(
    properties: Vec<String>,
    mut convert: impl FnMut(UiNodeId, String) -> UiPropertyMutationRequest,
) -> u128 {
    let started = Instant::now();
    for property in properties {
        black_box(convert(UiNodeId::new(89), black_box(property)));
    }
    started.elapsed().as_nanos()
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
