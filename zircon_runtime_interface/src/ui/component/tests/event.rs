use super::*;

#[test]
fn component_event_schema_names_round_trip_and_reject_unknown() {
    for kind in UiComponentEventKind::ALL {
        assert_eq!(
            UiComponentEventKind::from_schema_name(kind.schema_name()),
            Some(kind)
        );
    }
    assert_eq!(
        UiComponentEventKind::from_schema_name("value_changed"),
        None
    );
    assert_eq!(UiComponentEventKind::from_schema_name("Unknown"), None);
}

#[test]
#[ignore = "release-only component event schema-name dispatch benchmark"]
fn component_event_schema_name_dispatch_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const LOOKUP_COUNT: usize = 200_000;
    const SAMPLE_COUNT: usize = 11;
    let mut inputs = UiComponentEventKind::ALL
        .into_iter()
        .map(UiComponentEventKind::schema_name)
        .collect::<Vec<_>>();
    inputs.push("Unknown");

    let linear_lookup = |value: &str| {
        UiComponentEventKind::ALL
            .into_iter()
            .find(|kind| kind.schema_name() == value)
    };
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut direct_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for index in 0..LOOKUP_COUNT {
                black_box(linear_lookup(black_box(inputs[index % inputs.len()])));
            }
            started.elapsed().as_nanos()
        };
        let measure_direct = || {
            let started = Instant::now();
            for index in 0..LOOKUP_COUNT {
                black_box(UiComponentEventKind::from_schema_name(black_box(
                    inputs[index % inputs.len()],
                )));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            direct_samples.push(measure_direct());
        } else {
            direct_samples.push(measure_direct());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    direct_samples.sort_unstable();
    let p50 = SAMPLE_COUNT / 2;
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_COMPONENT_EVENT_SCHEMA_NAME_DISPATCH_BENCH_V1 lookups={LOOKUP_COUNT} variants={} samples={SAMPLE_COUNT} linear_p50_ns={} direct_p50_ns={} linear_p95_ns={} direct_p95_ns={}",
        inputs.len() - 1,
        linear_samples[p50],
        direct_samples[p50],
        linear_samples[p95],
        direct_samples[p95],
    );
    assert!(
        direct_samples[p95].saturating_mul(5) <= linear_samples[p95].saturating_mul(4),
        "direct component-event schema dispatch must improve P95 by at least 20%: linear={}ns direct={}ns",
        linear_samples[p95],
        direct_samples[p95],
    );
}
