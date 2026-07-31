use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::editing::engine::{HistoryContextId, TransactionId};
use crate::core::editor_message::{
    DocumentId, DocumentMessage, EditorMessage, EditorMessageBus, EditorMessageBusError,
    EditorMessageDispatchError, EditorMessageInboxLimits, EditorMessagePayload,
    EditorMessageRequest, EditorMessageResponse, EditorRequestHandler, EditorViewInvalidationMask,
    FocusMessage, ModeMessage, PlayStateKind, SceneInspectionFieldsDelta, SceneInspectionMessage,
    SceneModeId, SelectionDomain, TransactionMessage, TOPIC_SCENE_INSPECTION,
};

use super::fixture::{topic, view};

const MIXED_LOSSLESS_BACKLOG: u64 = 4_096;
const MAX_PUBLISH_P95_NS: u64 = 50_000_000;

#[test]
fn fanout_clones_share_one_immutable_delivery_payload() {
    let mut bus = EditorMessageBus::default();
    let first = bus.register_subscriber([topic("editor.focus")]).unwrap();
    let second = bus.register_subscriber([topic("editor.focus")]).unwrap();

    bus.publish(
        topic("editor.focus"),
        selection_changed(SelectionDomain::Scene, 1),
    );

    let first_deliveries = bus.deliveries_for(first);
    let second_deliveries = bus.deliveries_for(second);
    assert!(first_deliveries[0].shares_payload_with(&second_deliveries[0]));
}

#[test]
fn paused_hundred_subscriber_selection_storm_stays_one_delivery_per_inbox() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(4, 4, 4));
    let subscribers = (0..100)
        .map(|_| bus.register_subscriber([topic("editor.focus")]).unwrap())
        .collect::<Vec<_>>();

    for revision in 0..10_000 {
        bus.publish(
            topic("editor.focus"),
            selection_changed(SelectionDomain::Scene, revision),
        );
    }

    let first = bus.deliveries_for(subscribers[0])[0].clone();
    for subscriber in subscribers {
        let deliveries = bus.deliveries_for(subscriber);
        assert_eq!(deliveries.len(), 1);
        assert!(first.shares_payload_with(&deliveries[0]));
        let stats = bus.inbox_stats(subscriber).unwrap();
        assert_eq!(stats.depth(), 1);
        assert_eq!(stats.coalesced(), 9_999);
        assert_eq!(stats.dropped(), 0);
        assert_eq!(stats.backpressured(), 0);
    }
}

#[test]
fn latest_state_coalesces_but_document_edges_remain_ordered() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(4, 4, 4));
    let subscriber = bus
        .register_subscriber([topic("editor.focus"), topic("editor.document")])
        .unwrap();

    bus.publish(
        topic("editor.focus"),
        selection_changed(SelectionDomain::Scene, 1),
    );
    let coalesced = bus.publish(
        topic("editor.focus"),
        selection_changed(SelectionDomain::Scene, 2),
    );
    bus.publish(
        topic("editor.document"),
        document_opened(DocumentId::new(7)),
    );
    bus.publish(
        topic("editor.document"),
        document_opened(DocumentId::new(8)),
    );

    assert_eq!(coalesced.coalesced(), &[subscriber]);
    let deliveries = bus.deliveries_for(subscriber);
    assert_eq!(deliveries.len(), 3);
    assert_eq!(
        deliveries[0].message(),
        &selection_changed(SelectionDomain::Scene, 2)
    );
    assert_eq!(
        deliveries[1].message(),
        &document_opened(DocumentId::new(7))
    );
    assert_eq!(
        deliveries[2].message(),
        &document_opened(DocumentId::new(8))
    );
}

#[test]
fn bounded_custom_messages_evict_oldest_with_visible_pressure_stats() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(4, 2, 4));
    let subscriber = bus
        .register_subscriber([topic("plugin.telemetry")])
        .unwrap();

    for sequence in 0..3 {
        bus.publish(
            topic("plugin.telemetry"),
            EditorMessage::custom(
                "plugin.telemetry.v1",
                serde_json::json!({ "seq": sequence }),
            ),
        );
    }

    let deliveries = bus.deliveries_for(subscriber);
    assert_eq!(deliveries.len(), 2);
    assert_eq!(
        deliveries[0].message().payload(),
        &EditorMessagePayload::Custom {
            schema_id: "plugin.telemetry.v1".to_string(),
            payload: serde_json::json!({ "seq": 1 }),
        }
    );
    let stats = bus.inbox_stats(subscriber).expect("registered inbox");
    assert_eq!(stats.dropped(), 1);
    assert_eq!(stats.depth(), 2);
}

#[test]
fn lossless_overflow_backpressures_without_discarding_existing_edges() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(2, 2, 2));
    let subscriber = bus.register_subscriber([topic("editor.document")]).unwrap();

    bus.publish(
        topic("editor.document"),
        document_opened(DocumentId::new(1)),
    );
    bus.publish(
        topic("editor.document"),
        document_opened(DocumentId::new(2)),
    );
    let report = bus.publish(
        topic("editor.document"),
        document_opened(DocumentId::new(3)),
    );

    assert_eq!(report.backpressured(), &[subscriber]);
    assert!(report.delivered().is_empty());
    let deliveries = bus.deliveries_for(subscriber);
    assert_eq!(deliveries.len(), 2);
    assert_eq!(
        deliveries[0].message(),
        &document_opened(DocumentId::new(1))
    );
    assert_eq!(
        deliveries[1].message(),
        &document_opened(DocumentId::new(2))
    );
    assert_eq!(bus.inbox_stats(subscriber).unwrap().backpressured(), 1);
}

#[test]
fn lossless_fanout_rejects_every_subscriber_when_any_inbox_is_full() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(1, 1, 1));
    let full = bus
        .register_subscriber([topic("editor.transaction")])
        .unwrap();
    bus.publish(
        topic("editor.transaction"),
        EditorMessage::new(EditorMessagePayload::Transaction(
            TransactionMessage::Committed {
                transaction: TransactionId::from_sequence(1),
                history: HistoryContextId::Global,
                label: "first".to_string(),
                timestamp_frame: 0,
            },
        )),
    );
    let empty = bus
        .register_subscriber([topic("editor.transaction")])
        .unwrap();

    let report = bus.publish(
        topic("editor.transaction"),
        EditorMessage::new(EditorMessagePayload::Transaction(
            TransactionMessage::Committed {
                transaction: TransactionId::from_sequence(2),
                history: HistoryContextId::Global,
                label: "second".to_string(),
                timestamp_frame: 0,
            },
        )),
    );

    assert_eq!(report.backpressured(), &[full]);
    assert!(report.delivered().is_empty());
    assert_eq!(bus.deliveries_for(full).len(), 1);
    assert!(bus.deliveries_for(empty).is_empty());
}

#[test]
fn lossless_broadcast_rejects_every_subscriber_when_any_inbox_is_full() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(1, 1, 1));
    let full = bus.register_subscriber([topic("editor.document")]).unwrap();
    bus.publish(
        topic("editor.document"),
        document_opened(DocumentId::new(1)),
    );
    let empty = bus.register_subscriber([topic("editor.focus")]).unwrap();

    let report = bus.broadcast(
        topic("editor.transaction"),
        EditorMessage::new(EditorMessagePayload::Transaction(
            TransactionMessage::Committed {
                transaction: TransactionId::from_sequence(2),
                history: HistoryContextId::Global,
                label: "broadcast".to_string(),
                timestamp_frame: 0,
            },
        )),
    );

    assert_eq!(report.backpressured(), &[full]);
    assert!(report.delivered().is_empty());
    assert_eq!(bus.deliveries_for(full).len(), 1);
    assert!(bus.deliveries_for(empty).is_empty());
}

#[test]
fn request_backpressure_prevents_handler_execution() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(1, 1, 1));
    let subscriber = bus.register_subscriber([topic("editor.document")]).unwrap();
    bus.publish(
        topic("editor.document"),
        document_opened(DocumentId::new(1)),
    );
    let mut handler = CountingHandler::default();

    let error = bus
        .request(
            subscriber,
            topic("editor.document"),
            document_opened(DocumentId::new(2)),
            &mut handler,
        )
        .expect_err("a full lossless inbox must reject a synchronous request");

    assert_eq!(error, EditorMessageBusError::Backpressured { subscriber });
    assert_eq!(handler.calls, 0);
    assert_eq!(bus.deliveries_for(subscriber).len(), 1);
}

#[test]
fn play_state_edges_are_lossless_while_scene_mode_is_latest() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(4, 4, 4));
    let subscriber = bus.register_subscriber([topic("editor.mode")]).unwrap();

    bus.publish(
        topic("editor.mode"),
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
            from: PlayStateKind::Edit,
            to: PlayStateKind::Building,
        })),
    );
    bus.publish(
        topic("editor.mode"),
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::PlayStateChanged {
            from: PlayStateKind::Building,
            to: PlayStateKind::Playing,
        })),
    );

    assert_eq!(bus.deliveries_for(subscriber).len(), 2);
}

#[test]
fn mixed_lane_depths_bytes_and_drain_stay_consistent() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(2, 2, 2));
    let subscriber = bus.register_subscriber([topic("editor.mixed")]).unwrap();

    bus.publish(topic("editor.mixed"), document_opened(DocumentId::new(1)));
    bus.publish(
        topic("editor.mixed"),
        EditorMessage::custom("editor.mixed.v1", serde_json::json!({ "seq": 1 })),
    );
    bus.publish(
        topic("editor.mixed"),
        selection_changed(SelectionDomain::Scene, 1),
    );
    let initial = bus.inbox_stats(subscriber).unwrap();
    assert_eq!(initial.depth(), 3);
    assert_eq!(initial.lossless_depth(), 1);
    assert_eq!(initial.bounded_depth(), 1);
    assert_eq!(initial.latest_depth(), 1);
    assert_eq!(
        initial.depth(),
        initial.lossless_depth() + initial.bounded_depth() + initial.latest_depth()
    );
    assert!(initial.retained_bytes() > 0);

    bus.publish(
        topic("editor.mixed"),
        selection_changed(SelectionDomain::Scene, 2),
    );
    bus.publish(
        topic("editor.mixed"),
        EditorMessage::custom("editor.mixed.v1", serde_json::json!({ "seq": 2 })),
    );
    bus.publish(
        topic("editor.mixed"),
        EditorMessage::custom("editor.mixed.v1", serde_json::json!({ "seq": 3 })),
    );
    let pressured = bus.inbox_stats(subscriber).unwrap();
    assert_eq!(pressured.lossless_depth(), 1);
    assert_eq!(pressured.bounded_depth(), 2);
    assert_eq!(pressured.latest_depth(), 1);
    assert_eq!(pressured.coalesced(), 1);
    assert_eq!(pressured.dropped(), 1);

    let drained = bus.drain_deliveries(subscriber);
    assert_eq!(drained.len(), 4);
    let empty = bus.inbox_stats(subscriber).unwrap();
    assert_eq!(empty.depth(), 0);
    assert_eq!(empty.lossless_depth(), 0);
    assert_eq!(empty.bounded_depth(), 0);
    assert_eq!(empty.latest_depth(), 0);
    assert_eq!(empty.retained_bytes(), 0);
    assert_eq!(empty.drained(), 4);
}

#[test]
fn zero_capacity_and_byte_budget_reject_without_mutation() {
    let mut zero = EditorMessageBus::with_inbox_limits(
        EditorMessageInboxLimits::new(0, 0, 0).with_byte_limits(0, 0),
    );
    let subscriber = zero.register_subscriber([topic("editor.zero")]).unwrap();
    assert_eq!(
        zero.publish(topic("editor.zero"), document_opened(DocumentId::new(1)))
            .backpressured(),
        &[subscriber]
    );
    assert_eq!(
        zero.publish(
            topic("editor.zero"),
            selection_changed(SelectionDomain::Scene, 1),
        )
        .dropped(),
        &[subscriber]
    );
    assert_eq!(
        zero.publish(
            topic("editor.zero"),
            EditorMessage::custom("editor.zero.v1", serde_json::json!({ "value": 1 })),
        )
        .dropped(),
        &[subscriber]
    );
    let zero_stats = zero.inbox_stats(subscriber).unwrap();
    assert_eq!(zero_stats.depth(), 0);
    assert_eq!(zero_stats.backpressured(), 1);
    assert_eq!(zero_stats.dropped(), 2);

    let limits = EditorMessageInboxLimits::new(2, 2, 2).with_byte_limits(256, 512);
    let mut bytes = EditorMessageBus::with_inbox_limits(limits);
    let subscriber = bytes.register_subscriber([topic("editor.bytes")]).unwrap();
    let lossless = EditorMessage::new(EditorMessagePayload::Transaction(
        TransactionMessage::Committed {
            transaction: TransactionId::from_sequence(1),
            history: HistoryContextId::Global,
            label: "x".repeat(1_024),
            timestamp_frame: 0,
        },
    ));
    let latest = EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::SceneModeChanged {
        mode: SceneModeId::new("x".repeat(1_024)),
    }));
    let bounded = EditorMessage::custom(
        "editor.bytes.v1",
        serde_json::json!({ "blob": "x".repeat(1_024) }),
    );
    assert_eq!(
        bytes
            .publish(topic("editor.bytes"), lossless)
            .backpressured(),
        &[subscriber]
    );
    assert_eq!(
        bytes.publish(topic("editor.bytes"), latest).dropped(),
        &[subscriber]
    );
    assert_eq!(
        bytes.publish(topic("editor.bytes"), bounded).dropped(),
        &[subscriber]
    );
    let byte_stats = bytes.inbox_stats(subscriber).unwrap();
    assert_eq!(byte_stats.depth(), 0);
    assert_eq!(byte_stats.retained_bytes(), 0);
    assert_eq!(byte_stats.backpressured(), 1);
    assert_eq!(byte_stats.dropped(), 2);

    let mut total = EditorMessageBus::with_inbox_limits(
        EditorMessageInboxLimits::new(2, 2, 2).with_byte_limits(2_048, 768),
    );
    let subscriber = total.register_subscriber([topic("editor.total")]).unwrap();
    let first = total.publish(
        topic("editor.total"),
        EditorMessage::custom(
            "editor.total.v1",
            serde_json::json!({ "blob": "a".repeat(400) }),
        ),
    );
    assert_eq!(first.delivered(), &[subscriber]);
    let second = total.publish(
        topic("editor.total"),
        EditorMessage::custom(
            "editor.total.v1",
            serde_json::json!({ "blob": "b".repeat(400) }),
        ),
    );
    assert_eq!(second.delivered(), &[subscriber]);
    assert_eq!(second.dropped(), &[subscriber]);
    let total_stats = total.inbox_stats(subscriber).unwrap();
    assert_eq!(total_stats.depth(), 1);
    assert_eq!(total_stats.bounded_depth(), 1);
    assert_eq!(total_stats.dropped(), 1);
    assert!(total_stats.retained_bytes() <= 768);
}

#[test]
fn inspection_entity_deltas_count_toward_the_latest_message_byte_budget() {
    let mut bus = EditorMessageBus::with_inbox_limits(
        EditorMessageInboxLimits::new(2, 2, 2).with_byte_limits(4 * 1024, 8 * 1024),
    );
    let subscriber = bus
        .register_subscriber([topic(TOPIC_SCENE_INSPECTION)])
        .unwrap();
    let message = EditorMessage::new(EditorMessagePayload::SceneInspection(
        SceneInspectionMessage::delta(
            1,
            2,
            Some(1),
            (0..1_024).collect(),
            Vec::new(),
            Vec::new(),
            SceneInspectionFieldsDelta::unchanged(Some(1)),
        ),
    ));

    let report = bus.publish(topic(TOPIC_SCENE_INSPECTION), message);

    assert_eq!(report.dropped(), &[subscriber]);
    let stats = bus.inbox_stats(subscriber).unwrap();
    assert_eq!(stats.depth(), 0);
    assert_eq!(stats.latest_depth(), 0);
    assert_eq!(stats.retained_bytes(), 0);
    assert_eq!(stats.dropped(), 1);
}

#[test]
fn latest_replacement_evicts_other_latest_state_atomically_under_byte_pressure() {
    let mut bus = EditorMessageBus::with_inbox_limits(
        EditorMessageInboxLimits::new(2, 2, 3).with_byte_limits(2_048, 768),
    );
    let subscriber = bus.register_subscriber([topic("editor.latest")]).unwrap();

    bus.publish(
        topic("editor.latest"),
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::SceneModeChanged {
            mode: SceneModeId::new("a".repeat(400)),
        })),
    );
    bus.publish(
        topic("editor.latest"),
        selection_changed(SelectionDomain::Scene, 1),
    );

    let replacement =
        EditorMessage::new(EditorMessagePayload::Mode(ModeMessage::SceneModeChanged {
            mode: SceneModeId::new("b".repeat(600)),
        }));
    let report = bus.publish(topic("editor.latest"), replacement.clone());

    assert_eq!(report.delivered(), &[subscriber]);
    assert_eq!(report.coalesced(), &[subscriber]);
    assert_eq!(report.dropped(), &[subscriber]);
    let deliveries = bus.deliveries_for(subscriber);
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0].message(), &replacement);
    let stats = bus.inbox_stats(subscriber).unwrap();
    assert_eq!(stats.latest_depth(), 1);
    assert_eq!(stats.coalesced(), 1);
    assert_eq!(stats.dropped(), 1);
    assert!(stats.retained_bytes() <= 768);
}

#[test]
fn dirty_view_bytes_respect_delivery_budget_without_mutating_dirty_state() {
    let mut bus = EditorMessageBus::with_inbox_limits(
        EditorMessageInboxLimits::new(2, 2, 2).with_byte_limits(256, 512),
    );
    let subscriber = bus.register_subscriber([topic("editor.dirty")]).unwrap();
    let latest_view = view(&"latest".repeat(256));
    let lossless_view = view(&"lossless".repeat(256));

    let latest = selection_changed(SelectionDomain::Scene, 1).with_dirty(
        latest_view.clone(),
        EditorViewInvalidationMask::PRESENTATION_DATA,
    );
    let latest_report = bus.publish(topic("editor.dirty"), latest);
    assert_eq!(latest_report.dropped(), &[subscriber]);

    let lossless = document_opened(DocumentId::new(1)).with_dirty(
        lossless_view.clone(),
        EditorViewInvalidationMask::PRESENTATION_DATA,
    );
    let lossless_report = bus.publish(topic("editor.dirty"), lossless);
    assert_eq!(lossless_report.backpressured(), &[subscriber]);

    let stats = bus.inbox_stats(subscriber).unwrap();
    assert_eq!(stats.depth(), 0);
    assert_eq!(stats.retained_bytes(), 0);
    assert_eq!(stats.dropped(), 1);
    assert_eq!(stats.backpressured(), 1);
    assert!(bus.dirty_set().mask_for(&latest_view).is_none());
    assert!(bus.dirty_set().mask_for(&lossless_view).is_none());
}

#[test]
fn identifier_exhaustion_is_typed_and_atomic() {
    let mut bus = EditorMessageBus::default();
    bus.set_next_subscriber_id_for_test(u64::MAX - 1);
    let subscriber = bus
        .register_subscriber([topic("editor.exhaustion")])
        .unwrap();
    assert_eq!(subscriber.value(), u64::MAX);
    assert_eq!(
        bus.register_subscriber([topic("editor.other")]),
        Err(EditorMessageBusError::SubscriberIdExhausted)
    );

    bus.publish(
        topic("editor.exhaustion"),
        document_opened(DocumentId::new(1)),
    );
    assert_eq!(bus.deliveries_for(subscriber).len(), 1);
    let dirty_view = view("editor.exhaustion.view");
    bus.set_next_delivery_sequence_for_test(u64::MAX);
    let failed = bus.publish(
        topic("editor.exhaustion"),
        selection_changed(SelectionDomain::Scene, 2).with_dirty(
            dirty_view.clone(),
            EditorViewInvalidationMask::PRESENTATION_DATA,
        ),
    );
    assert_eq!(
        failed.error(),
        Some(EditorMessageDispatchError::DeliverySequenceExhausted)
    );
    assert_eq!(bus.deliveries_for(subscriber).len(), 1);
    assert!(bus.dirty_set().mask_for(&dirty_view).is_none());

    let broadcast = bus.broadcast(
        topic("editor.exhaustion"),
        document_opened(DocumentId::new(2)),
    );
    assert_eq!(
        broadcast.error(),
        Some(EditorMessageDispatchError::DeliverySequenceExhausted)
    );
    let mut handler = CountingHandler::default();
    assert_eq!(
        bus.request(
            subscriber,
            topic("editor.exhaustion"),
            document_opened(DocumentId::new(3)),
            &mut handler,
        ),
        Err(EditorMessageBusError::DeliverySequenceExhausted)
    );
    assert_eq!(handler.calls, 0);
    assert_eq!(bus.deliveries_for(subscriber).len(), 1);
}

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
        selection_changed(SelectionDomain::Scene, 0),
    );

    let large_message = EditorMessage::custom(
        "editor.benchmark.large-json.v1",
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
        selection_changed(SelectionDomain::Scene, 0),
    );

    let mut publish_durations = Vec::with_capacity(STORM_PUBLISHES as usize);
    let mut steady_allocations = AllocationSample::default();
    let rss_before = working_set_bytes();
    for revision in 1..=STORM_PUBLISHES {
        let message = selection_changed(SelectionDomain::Scene, revision);
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

fn selection_changed(domain: SelectionDomain, revision: u64) -> EditorMessage {
    EditorMessage::new(EditorMessagePayload::Focus(
        FocusMessage::SelectionChanged { domain, revision },
    ))
}

fn document_opened(doc: DocumentId) -> EditorMessage {
    EditorMessage::new(EditorMessagePayload::Document(DocumentMessage::Opened {
        doc,
    }))
}

#[derive(Default)]
struct CountingHandler {
    calls: usize,
}

impl EditorRequestHandler for CountingHandler {
    fn handle_editor_request(&mut self, _request: &EditorMessageRequest) -> EditorMessageResponse {
        self.calls += 1;
        EditorMessageResponse::handled(document_opened(DocumentId::new(99)))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct AllocationSample {
    operations: u64,
    bytes: u64,
}

impl AllocationSample {
    fn accumulate(&mut self, sample: Self) {
        self.operations = self.operations.saturating_add(sample.operations);
        self.bytes = self.bytes.saturating_add(sample.bytes);
    }
}

fn measure_allocations<T>(operation: impl FnOnce() -> T) -> (T, Duration, AllocationSample) {
    let tracking = AllocationTrackingGuard::start();
    let started = Instant::now();
    let output = operation();
    let elapsed = started.elapsed();
    drop(tracking);
    let sample = AllocationSample {
        operations: TRACKED_ALLOCATION_OPERATIONS.load(Ordering::Relaxed),
        bytes: TRACKED_ALLOCATION_BYTES.load(Ordering::Relaxed),
    };
    (output, elapsed, sample)
}

struct AllocationTrackingGuard;

impl AllocationTrackingGuard {
    fn start() -> Self {
        TRACKED_ALLOCATION_OPERATIONS.store(0, Ordering::Relaxed);
        TRACKED_ALLOCATION_BYTES.store(0, Ordering::Relaxed);
        TRACK_ALLOCATIONS.store(true, Ordering::Release);
        Self
    }
}

impl Drop for AllocationTrackingGuard {
    fn drop(&mut self) {
        TRACK_ALLOCATIONS.store(false, Ordering::Release);
    }
}

fn record_allocation(size: usize, pointer_is_valid: bool) {
    if pointer_is_valid && TRACK_ALLOCATIONS.load(Ordering::Acquire) {
        TRACKED_ALLOCATION_OPERATIONS.fetch_add(1, Ordering::Relaxed);
        TRACKED_ALLOCATION_BYTES
            .fetch_add(u64::try_from(size).unwrap_or(u64::MAX), Ordering::Relaxed);
    }
}

struct TrackingAllocator;

// SAFETY: every operation delegates to `System` with the original pointer/layout contract;
// the additional relaxed atomics do not retain or modify allocation addresses.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: delegated with the caller-provided valid allocation layout.
        let pointer = unsafe { System.alloc(layout) };
        record_allocation(layout.size(), !pointer.is_null());
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: delegated with the caller-provided valid allocation layout.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record_allocation(layout.size(), !pointer.is_null());
        pointer
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: delegated with the caller-provided pointer, original layout, and new size.
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        record_allocation(new_size, !new_pointer.is_null());
        new_pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: delegated with the exact pointer/layout pair supplied by the caller.
        unsafe { System.dealloc(pointer, layout) };
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator;
static TRACK_ALLOCATIONS: AtomicBool = AtomicBool::new(false);
static TRACKED_ALLOCATION_OPERATIONS: AtomicU64 = AtomicU64::new(0);
static TRACKED_ALLOCATION_BYTES: AtomicU64 = AtomicU64::new(0);

#[cfg(windows)]
fn working_set_bytes() -> Option<u64> {
    let command = format!("(Get-Process -Id {}).WorkingSet64", std::process::id());
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &command])
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    String::from_utf8(output.stdout).ok()?.trim().parse().ok()
}

#[cfg(not(windows))]
fn working_set_bytes() -> Option<u64> {
    None
}
