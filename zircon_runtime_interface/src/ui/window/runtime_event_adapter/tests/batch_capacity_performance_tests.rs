use super::*;

fn push_events_default(
    events: impl IntoIterator<Item = UiWindowInputPumpEvent>,
) -> UiWindowInputPumpBatch {
    let mut batch = UiWindowInputPumpBatch::default();
    for event in events {
        batch.push(event);
    }
    batch
}

fn push_events_reserved(
    events: impl IntoIterator<Item = UiWindowInputPumpEvent>,
) -> UiWindowInputPumpBatch {
    let events = events.into_iter();
    let mut batch = UiWindowInputPumpBatch::with_capacity(events.size_hint().0);
    for event in events {
        batch.push(event);
    }
    batch
}

fn sample_event() -> UiWindowInputPumpEvent {
    UiWindowInputPumpEvent::Window(UiWindowEvent::window_close(UiWindowEventMetadata::default()))
}

fn sample_events() -> impl Iterator<Item = UiWindowInputPumpEvent> {
    std::iter::repeat_with(sample_event).take(256)
}

#[test]
fn runtime_interface03_batch55_61_reserved_input_batch_preserves_events() {
    let default_batch = push_events_default(sample_events());
    let reserved_batch = push_events_reserved(sample_events());

    assert_eq!(reserved_batch, default_batch);
    assert_eq!(reserved_batch.events.len(), 256);
}

#[test]
#[ignore = "release-only reserved input batch benchmark"]
fn runtime_interface03_batch55_61_window_input_batch_capacity_release_benchmark() {
    use std::{hint::black_box, time::Instant};

    const BUILD_COUNT: usize = 25_000;
    const EVENT_COUNT: usize = 256;
    const SAMPLE_COUNT: usize = 11;
    let mut default_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_default = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(push_events_default(black_box(sample_events())));
            }
            started.elapsed().as_nanos()
        };
        let measure_reserved = || {
            let started = Instant::now();
            for _ in 0..BUILD_COUNT {
                black_box(push_events_reserved(black_box(sample_events())));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            default_samples.push(measure_default());
            reserved_samples.push(measure_reserved());
        } else {
            reserved_samples.push(measure_reserved());
            default_samples.push(measure_default());
        }
    }

    default_samples.sort_unstable();
    reserved_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BATCH_CAPACITY_BENCH_V1 builds={BUILD_COUNT} events={} samples={SAMPLE_COUNT} default_p95_ns={} reserved_p95_ns={}",
        EVENT_COUNT, default_samples[p95], reserved_samples[p95],
    );
    assert!(
        reserved_samples[p95].saturating_mul(2) <= default_samples[p95],
        "reserved input batches must improve P95 by at least 50%: default={}ns reserved={}ns",
        default_samples[p95],
        reserved_samples[p95],
    );
}
