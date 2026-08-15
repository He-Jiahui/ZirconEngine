//! Backpressure behavior matrix for the editor message bus.

use crate::core::editing::engine::{HistoryContextId, TransactionId};
use crate::core::editor_message::{
    DocumentId, EditorMessage, EditorMessageBus, EditorMessageBusError, EditorMessageDispatchError,
    EditorMessageInboxLimits, EditorMessagePayload, EditorViewInvalidationMask, ModeMessage,
    PlayStateKind, SceneInspectionFieldsDelta, SceneInspectionHierarchyAnchor,
    SceneInspectionMessage, SceneInspectionSelectionDelta, SceneModeId, SelectionDomain,
    SharedEditorMessageBus, TransactionMessage, TOPIC_SCENE_INSPECTION,
};

use super::super::fixture::{topic, view};
use super::fixture::{document_opened, selection_changed, CountingHandler, PayloadSharingHandler};

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
fn shared_request_reuses_the_enqueued_delivery_payload() {
    let bus = SharedEditorMessageBus::default();
    let target = bus
        .register_subscriber([topic("editor.request.payload")])
        .unwrap();
    let message = EditorMessage::custom(
        "editor.request.payload.v1",
        serde_json::json!({ "blob": "x".repeat(64 * 1024) }),
    );
    let mut handler = PayloadSharingHandler::default();

    bus.request(
        target,
        topic("editor.request.payload"),
        message,
        &mut handler,
    )
    .unwrap();

    let delivery = bus
        .deliveries_for(target)
        .pop()
        .expect("request was enqueued");
    assert!(handler
        .request
        .as_ref()
        .expect("handler observed the request")
        .shares_payload_with(&delivery));
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
fn latest_scene_inspection_composes_selection_from_the_oldest_retained_revision() {
    let mut bus = EditorMessageBus::with_inbox_limits(EditorMessageInboxLimits::new(4, 4, 4));
    let subscriber = bus
        .register_subscriber([topic(TOPIC_SCENE_INSPECTION)])
        .unwrap();

    for selection in [
        SceneInspectionSelectionDelta::between(4, 5, vec![7, 9], vec![3]),
        SceneInspectionSelectionDelta::between(5, 6, vec![3, 11], vec![9]),
    ] {
        bus.publish(
            topic(TOPIC_SCENE_INSPECTION),
            EditorMessage::new(EditorMessagePayload::SceneInspection(
                SceneInspectionMessage::delta(
                    20,
                    20,
                    None,
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    false,
                    SceneInspectionFieldsDelta::unchanged(None),
                    selection,
                ),
            )),
        );
    }

    let deliveries = bus.deliveries_for(subscriber);
    assert_eq!(deliveries.len(), 1);
    let EditorMessagePayload::SceneInspection(message) = deliveries[0].message().payload() else {
        panic!("latest delivery should retain the scene inspection message");
    };
    assert_eq!(message.selection().previous_revision(), Some(4));
    assert_eq!(message.selection().revision(), 6);
    assert_eq!(message.selection().added_entities(), &[7, 11]);
    assert!(message.selection().removed_entities().is_empty());
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
            (0..1_024)
                .map(|entity| SceneInspectionHierarchyAnchor::new(entity, None, 0, entity))
                .collect(),
            Vec::new(),
            Vec::new(),
            false,
            SceneInspectionFieldsDelta::unchanged(Some(1)),
            crate::core::editor_message::SceneInspectionSelectionDelta::unchanged(),
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
