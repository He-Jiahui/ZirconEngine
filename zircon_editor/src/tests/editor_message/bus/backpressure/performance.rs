use crate::core::editor_message::{
    DocumentId, EditorMessage, EditorMessageBus, EditorMessageSchemaId, SelectionDomain,
};

use super::super::fixture::topic;
use super::fixture::{
    document_opened, measure_allocations, selection_changed, working_set_bytes, AllocationSample,
    MAX_PUBLISH_P95_NS, MIXED_LOSSLESS_BACKLOG,
};

#[test]
#[ignore = "managed performance evidence; run alone with --test-threads=1"]
fn managed_fanout_allocation_rss_queue_age_and_publish_p95_report() {
    let reports = [1, 5, 100]
        .into_iter()
        .map(run_fanout_benchmark)
        .collect::<Vec<_>>();

    println!(
        "EDITOR02_FANOUT_BENCHMARK={}",
        serde_json::Value::Array(reports)
    );
}

#[test]
#[ignore = "managed performance evidence; run alone with --test-threads=1"]
fn optimization_wave_20260824_editor48_zero_route_publish_allocation_evidence() {
    const PUBLISHES: u64 = 100_000;
    const MAX_ELAPSED_NS: u128 = 500_000_000;

    let mut bus = EditorMessageBus::default();
    let publish_topic = topic("editor.unrouted-benchmark");
    let message = selection_changed(SelectionDomain::edit_scene(), 0);
    let (reports, elapsed, allocations) = measure_allocations(|| {
        let mut reports = 0_u64;
        for _ in 0..PUBLISHES {
            let report = bus.publish(publish_topic.clone(), message.clone());
            reports = reports.saturating_add(u64::from(
                report.delivered().is_empty() && report.error().is_none(),
            ));
        }
        reports
    });

    assert_eq!(reports, PUBLISHES);
    assert!(
        allocations.operations <= PUBLISHES.saturating_add(16),
        "zero-route publish should allocate only the owned topic clone: {allocations:?}"
    );
    assert!(
        elapsed.as_nanos() <= MAX_ELAPSED_NS,
        "zero-route publish elapsed {}ns exceeds {MAX_ELAPSED_NS}ns",
        elapsed.as_nanos()
    );

    let legacy_minimum_allocations = PUBLISHES.saturating_mul(2);
    let allocation_reduction_bps = legacy_minimum_allocations
        .saturating_sub(allocations.operations)
        .saturating_mul(10_000)
        / legacy_minimum_allocations;
    println!(
        "EDITOR_MESSAGE_NO_ROUTE_BENCH_V1 publishes={PUBLISHES} legacy_min_alloc_ops={legacy_minimum_allocations} optimized_alloc_ops={} optimized_alloc_bytes={} allocation_reduction_bps={allocation_reduction_bps} elapsed_ns={} max_elapsed_ns={MAX_ELAPSED_NS}",
        allocations.operations,
        allocations.bytes,
        elapsed.as_nanos()
    );
}

fn run_fanout_benchmark(subscriber_count: usize) -> serde_json::Value {
    const LARGE_PAYLOAD_BYTES: usize = 1024 * 1024;
    const STORM_PUBLISHES: u64 = 10_000;

    let benchmark_topic = topic("editor.benchmark");
    let mut payload_bus = EditorMessageBus::default();
    let payload_subscribers = (0..subscriber_count)
        .map(|_| {
            payload_bus
                .register_subscriber([benchmark_topic.clone()])
                .unwrap()
        })
        .collect::<Vec<_>>();
    payload_bus.publish(
        benchmark_topic.clone(),
        selection_changed(SelectionDomain::edit_scene(), 0),
    );

    let large_message = EditorMessage::custom(
        EditorMessageSchemaId::editor("benchmark.large-json.v1").unwrap(),
        serde_json::json!({ "blob": "x".repeat(LARGE_PAYLOAD_BYTES) }),
    );
    let (large_report, large_elapsed, large_allocations) =
        measure_allocations(|| payload_bus.publish(benchmark_topic.clone(), large_message));
    assert_eq!(large_report.delivered().len(), subscriber_count);
    assert!(large_report.dropped().is_empty());
    assert!(large_report.backpressured().is_empty());
    let metadata_byte_budget = LARGE_PAYLOAD_BYTES / 8 + subscriber_count.saturating_mul(4_096);
    assert!(
        large_allocations.bytes < u64::try_from(metadata_byte_budget).unwrap_or(u64::MAX),
        "publication must not deep-clone the 1 MiB JSON payload for each subscriber: {large_allocations:?}"
    );
    let metadata_operation_budget = subscriber_count.saturating_mul(4).saturating_add(16);
    assert!(
        large_allocations.operations
            <= u64::try_from(metadata_operation_budget).unwrap_or(u64::MAX),
        "fanout may allocate bounded per-inbox metadata but must not clone payload data: {large_allocations:?}"
    );
    for subscriber in payload_subscribers {
        assert_eq!(payload_bus.inbox_stats(subscriber).unwrap().depth(), 2);
    }

    let mut storm_bus = EditorMessageBus::default();
    let storm_subscribers = (0..subscriber_count)
        .map(|_| {
            storm_bus
                .register_subscriber([benchmark_topic.clone()])
                .unwrap()
        })
        .collect::<Vec<_>>();
    for revision in 0..MIXED_LOSSLESS_BACKLOG {
        let report = storm_bus.publish(
            benchmark_topic.clone(),
            document_opened(DocumentId::new(revision)),
        );
        assert_eq!(report.delivered().len(), subscriber_count);
        assert!(report.backpressured().is_empty());
    }
    storm_bus.publish(
        benchmark_topic.clone(),
        selection_changed(SelectionDomain::edit_scene(), 0),
    );

    let mut publish_durations = Vec::with_capacity(STORM_PUBLISHES as usize);
    let mut steady_allocations = AllocationSample::default();
    let rss_before = working_set_bytes();
    for revision in 1..=STORM_PUBLISHES {
        let message = selection_changed(SelectionDomain::edit_scene(), revision);
        let publish_topic = benchmark_topic.clone();
        let (report, elapsed, allocations) =
            measure_allocations(|| storm_bus.publish(publish_topic, message));
        assert_eq!(report.delivered().len(), subscriber_count);
        assert_eq!(report.coalesced().len(), subscriber_count);
        assert!(report.dropped().is_empty());
        assert!(report.backpressured().is_empty());
        publish_durations.push(elapsed);
        steady_allocations.accumulate(allocations);
    }
    let rss_after = working_set_bytes();
    #[cfg(windows)]
    {
        assert!(rss_before.is_some(), "Windows RSS pre-sample must succeed");
        assert!(rss_after.is_some(), "Windows RSS post-sample must succeed");
    }

    publish_durations.sort_unstable();
    let p95_index = publish_durations.len().saturating_mul(95).div_ceil(100) - 1;
    let publish_p95_ns = u64::try_from(publish_durations[p95_index].as_nanos()).unwrap_or(u64::MAX);
    assert!(
        publish_p95_ns <= MAX_PUBLISH_P95_NS,
        "mixed-backlog publish p95 {publish_p95_ns}ns exceeds {MAX_PUBLISH_P95_NS}ns"
    );
    let first_stats = storm_bus
        .inbox_stats(storm_subscribers[0])
        .expect("registered subscriber inbox");
    assert_eq!(
        first_stats.depth(),
        usize::try_from(MIXED_LOSSLESS_BACKLOG).unwrap() + 1
    );
    assert_eq!(
        first_stats.lossless_depth(),
        usize::try_from(MIXED_LOSSLESS_BACKLOG).unwrap()
    );
    assert_eq!(first_stats.latest_depth(), 1);
    assert_eq!(first_stats.coalesced(), STORM_PUBLISHES);
    assert_eq!(first_stats.dropped(), 0);
    assert_eq!(first_stats.backpressured(), 0);
    assert!(first_stats.age_in_messages() >= MIXED_LOSSLESS_BACKLOG);
    for subscriber in storm_subscribers.into_iter().skip(1) {
        assert_eq!(storm_bus.inbox_stats(subscriber), Some(first_stats));
    }

    let rss_growth_bytes = rss_before.zip(rss_after).map(|(before, after)| {
        i64::try_from(after).unwrap_or(i64::MAX) - i64::try_from(before).unwrap_or(i64::MAX)
    });
    if let Some(growth) = rss_growth_bytes {
        assert!(
            growth <= 64 * 1024 * 1024,
            "paused latest-state storm retained too much process memory: {growth} bytes"
        );
    }

    serde_json::json!({
        "subscribers": subscriber_count,
        "large_payload_bytes": LARGE_PAYLOAD_BYTES,
        "large_payload_publish_ns": u64::try_from(large_elapsed.as_nanos()).unwrap_or(u64::MAX),
        "large_payload_publish_allocations": large_allocations.operations,
        "large_payload_publish_allocated_bytes": large_allocations.bytes,
        "storm_publishes": STORM_PUBLISHES,
        "mixed_lossless_backlog": MIXED_LOSSLESS_BACKLOG,
        "max_publish_p95_ns": MAX_PUBLISH_P95_NS,
        "steady_publish_allocations_total": steady_allocations.operations,
        "steady_publish_allocated_bytes_total": steady_allocations.bytes,
        "publish_p95_ns": publish_p95_ns,
        "rss_before_bytes": rss_before,
        "rss_after_bytes": rss_after,
        "rss_growth_bytes": rss_growth_bytes,
        "queue_depth": first_stats.depth(),
        "queue_age_messages": first_stats.age_in_messages(),
        "coalesced": first_stats.coalesced(),
        "dropped": first_stats.dropped(),
        "backpressured": first_stats.backpressured(),
    })
}
