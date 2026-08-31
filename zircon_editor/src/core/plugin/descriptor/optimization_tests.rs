use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime::plugin::PluginEventConsumerManifest;

use super::EditorPluginDescriptor;

const SAMPLE_COUNT: usize = 17;
const ITERATIONS: usize = 64;
const CONSUMER_COUNT: usize = 2_048;

fn percentile_95(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100) - 1]
}

fn consumer_fixture() -> Vec<PluginEventConsumerManifest> {
    (0..CONSUMER_COUNT)
        .rev()
        .map(|index| {
            PluginEventConsumerManifest::new(
                format!("consumer.{index:04}"),
                format!("event.{index:04}"),
                "fixture.schema",
            )
        })
        .collect()
}

fn legacy_descriptor(consumers: &[PluginEventConsumerManifest]) -> EditorPluginDescriptor {
    let mut descriptor = EditorPluginDescriptor::new("plugin.fixture", "Fixture", "fixture");
    for consumer in consumers {
        descriptor.event_consumers.push(consumer.clone());
        descriptor
            .event_consumers
            .sort_by(|left, right| left.consumer_id.cmp(&right.consumer_id));
    }
    descriptor
}

fn optimized_descriptor(consumers: &[PluginEventConsumerManifest]) -> EditorPluginDescriptor {
    consumers.iter().cloned().fold(
        EditorPluginDescriptor::new("plugin.fixture", "Fixture", "fixture"),
        |descriptor, consumer| descriptor.with_event_consumer(consumer),
    )
}

#[test]
fn editor06_descriptor_lifecycle_event_consumer_order() {
    let descriptor = optimized_descriptor(&consumer_fixture());
    let ids = descriptor
        .event_consumers
        .iter()
        .map(|consumer| consumer.consumer_id.as_str())
        .collect::<Vec<_>>();
    assert!(ids.windows(2).all(|window| window[0] <= window[1]));
    assert_eq!(ids.len(), CONSUMER_COUNT);
}

#[test]
fn editor06_descriptor_lifecycle_event_consumer_source_contract() {
    let source = include_str!("../descriptor.rs");
    assert!(source.contains("sort_unstable_by(|left, right|"));
    assert!(!source.contains("sort_by(|left, right|"));
}

#[test]
#[ignore = "Windows-native release performance evidence"]
fn editor06_descriptor_lifecycle_event_consumer_bench() {
    let consumers = consumer_fixture();
    let legacy = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_descriptor(&consumers));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let optimized = (0..SAMPLE_COUNT)
        .map(|_| {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(optimized_descriptor(&consumers));
            }
            started.elapsed()
        })
        .collect::<Vec<_>>();
    let legacy_p95 = percentile_95(legacy);
    let optimized_p95 = percentile_95(optimized);
    println!(
        "EDITOR06_UNSTABLE_EVENT_CONSUMER_SORT_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} consumers={} stable_sorts=1->0",
        legacy_p95.as_nanos(),
        optimized_p95.as_nanos(),
        SAMPLE_COUNT,
        ITERATIONS,
        CONSUMER_COUNT,
    );
    assert_eq!(
        optimized_descriptor(&consumers).event_consumers.len(),
        CONSUMER_COUNT
    );
    assert!(
        optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 95,
        "optimized p95 should be at most 95% of legacy p95"
    );
}
