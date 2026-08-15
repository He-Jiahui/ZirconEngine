use std::time::Duration;

use serde_json::json;

use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventId, EditorEventListenerControlRequest,
    EditorEventRecord, EditorEventResult, EditorEventRetentionBudget, EditorEventRetentionBudgets,
    EditorEventRetentionPolicy, EditorEventSequence, EditorEventService, EditorEventSource,
    EditorEventTransient, EditorEventUndoPolicy, EditorOperationEvent, EditorViewportEvent,
};
use crate::core::editor_message::SharedEditorMessageBus;

fn budget(max_records: usize, max_bytes: usize) -> EditorEventRetentionBudget {
    EditorEventRetentionBudget::new(max_records, max_bytes, Duration::from_secs(60))
        .expect("valid retention budget")
}

fn policy(
    journal: (usize, usize, usize),
    listeners: (usize, usize, usize),
) -> EditorEventRetentionPolicy {
    EditorEventRetentionPolicy::new(
        EditorEventRetentionBudgets::new(
            budget(journal.0, 1024 * 1024),
            budget(journal.1, 1024 * 1024),
            budget(journal.2, 1024 * 1024),
        ),
        EditorEventRetentionBudgets::new(
            budget(listeners.0, 1024 * 1024),
            budget(listeners.1, 1024 * 1024),
            budget(listeners.2, 1024 * 1024),
        ),
    )
}

fn record(sequence: u64, event: EditorEvent) -> EditorEventRecord {
    EditorEventRecord {
        event_id: EditorEventId::new(sequence),
        sequence: EditorEventSequence::new(sequence),
        source: EditorEventSource::Headless,
        event,
        binding_path: None,
        operation_id: None,
        operation_display_name: None,
        operation_arguments: None,
        operation_group: None,
        transaction_id: None,
        save_generation: None,
        effects: Vec::<EditorEventEffect>::new(),
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: sequence.saturating_sub(1),
        after_revision: sequence,
        result: EditorEventResult::success(json!({ "sequence": sequence })),
    }
}

fn durable_event(sequence: u64) -> EditorEvent {
    EditorEvent::Operation(EditorOperationEvent::CommandExecuted {
        operation_id: format!("scene.node.create_{sequence}"),
        transaction_id: sequence,
        group_open: false,
    })
}

#[test]
fn journal_enforces_independent_retention_classes_and_preserves_sequence_order() {
    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        policy((4, 2, 1), (8, 8, 8)),
    );

    for sequence in 1..=10 {
        service.record(record(sequence, durable_event(sequence)));
    }
    service.record(record(
        11,
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 1.0, y: 2.0 }),
    ));
    service.record(record(
        12,
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 3.0, y: 4.0 }),
    ));
    for sequence in 13..=15 {
        service.record(record(
            sequence,
            EditorEvent::Viewport(EditorViewportEvent::Scrolled { delta: 1.0 }),
        ));
    }

    let journal = service.journal();
    assert_eq!(
        journal
            .records()
            .iter()
            .map(|record| record.sequence.0)
            .collect::<Vec<_>>(),
        vec![7, 8, 9, 10, 12, 14, 15]
    );
    let diagnostics = journal.retention_diagnostics();
    assert_eq!(diagnostics.durable_replay.retained_records, 4);
    assert_eq!(diagnostics.durable_replay.dropped_records, 6);
    assert_eq!(diagnostics.latest_state.retained_records, 1);
    assert_eq!(diagnostics.latest_state.coalesced_records, 1);
    assert_eq!(diagnostics.frame_local.retained_records, 2);
    assert_eq!(diagnostics.frame_local.dropped_records, 1);
}

#[test]
fn paused_listener_storm_is_bounded_and_reports_the_exact_lag_window() {
    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        policy((128, 8, 8), (64, 8, 4)),
    );
    let listener_id = "External.PausedHistory".to_string();
    let registered =
        service.handle_listener_control_request(EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Paused History".to_string(),
        });
    assert!(registered.error.is_none());

    for sequence in 1..=10_000 {
        service.record(record(sequence, durable_event(sequence)));
    }

    let status = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryListenerStatus {
            listener_id: listener_id.clone(),
        },
    );
    assert!(status.error.is_none());
    assert_eq!(status.value["pending_delivery_count"], 64);
    assert_eq!(status.value["first_pending_sequence"], 9_937);
    assert_eq!(status.value["last_pending_sequence"], 10_000);
    assert_eq!(status.value["dropped_delivery_count"], 9_936);
    assert_eq!(status.value["lagged_since_sequence"], 1);
    assert_eq!(status.value["last_dropped_sequence"], 9_936);

    let deliveries = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id,
            after_delivery_cursor: 0,
            max_deliveries: 64,
        },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 64);
    assert_eq!(deliveries.first().unwrap()["sequence"], 9_937);
    assert_eq!(deliveries.last().unwrap()["sequence"], 10_000);

    let journal = service.journal();
    assert_eq!(journal.records().len(), 128);
    assert_eq!(
        journal.records().first().unwrap().sequence.0,
        10_000 - 128 + 1
    );
}

#[test]
fn immutable_routes_keep_one_thousand_listener_inboxes_bounded_and_cursor_addressable() {
    const LISTENER_COUNT: usize = 1_000;
    const EVENT_COUNT: u64 = 1_000;
    const RETAINED_PER_LISTENER: u64 = 32;

    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        policy((128, 8, 8), (RETAINED_PER_LISTENER as usize, 8, 8)),
    );
    for listener_index in 0..LISTENER_COUNT {
        let response =
            service.handle_listener_control_request(EditorEventListenerControlRequest::Register {
                listener_id: format!("External.Fanout.{listener_index}"),
                display_name: format!("Fanout {listener_index}"),
            });
        assert!(response.error.is_none());
    }

    for sequence in 1..=EVENT_COUNT {
        service.record(record(sequence, durable_event(sequence)));
    }

    for listener_index in 0..LISTENER_COUNT {
        let listener_id = format!("External.Fanout.{listener_index}");
        let status = service.handle_listener_control_request(
            EditorEventListenerControlRequest::QueryListenerStatus { listener_id },
        );
        assert!(status.error.is_none());
        assert_eq!(
            status.value["pending_delivery_count"],
            RETAINED_PER_LISTENER
        );
        assert_eq!(
            status.value["first_pending_sequence"],
            EVENT_COUNT - RETAINED_PER_LISTENER + 1
        );
        assert_eq!(status.value["last_pending_sequence"], EVENT_COUNT);
        assert_eq!(
            status.value["dropped_delivery_count"],
            EVENT_COUNT - RETAINED_PER_LISTENER
        );
    }

    for listener_index in [0, LISTENER_COUNT / 2, LISTENER_COUNT - 1] {
        let listener_id = format!("External.Fanout.{listener_index}");
        let page = service.handle_listener_control_request(
            EditorEventListenerControlRequest::QueryDeliveriesPage {
                listener_id,
                after_delivery_cursor: 0,
                max_deliveries: 4,
            },
        );
        let deliveries = page.value["deliveries"].as_array().expect("deliveries");
        assert_eq!(deliveries.len(), 4);
        assert_eq!(
            deliveries[0]["sequence"],
            EVENT_COUNT - RETAINED_PER_LISTENER + 1
        );
        assert_eq!(
            deliveries[3]["sequence"],
            EVENT_COUNT - RETAINED_PER_LISTENER + 4
        );
        assert_eq!(
            page.value["next_delivery_cursor"],
            EVENT_COUNT - RETAINED_PER_LISTENER + 4
        );
        assert_eq!(page.value["has_more"], true);
    }
}

#[test]
fn latest_state_storm_coalesces_without_marking_listener_lagged() {
    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        policy((8, 8, 1), (8, 8, 1)),
    );
    let listener_id = "External.ViewportState".to_string();
    service.handle_listener_control_request(EditorEventListenerControlRequest::Register {
        listener_id: listener_id.clone(),
        display_name: "Viewport State".to_string(),
    });

    for sequence in 1..=1_000 {
        service.record(record(
            sequence,
            EditorEvent::Viewport(EditorViewportEvent::PointerMoved {
                x: sequence as f32,
                y: 0.0,
            }),
        ));
    }

    let status = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryListenerStatus { listener_id },
    );
    assert_eq!(status.value["pending_delivery_count"], 1);
    assert_eq!(status.value["first_pending_sequence"], 1_000);
    assert_eq!(status.value["last_pending_sequence"], 1_000);
    assert_eq!(status.value["coalesced_delivery_count"], 999);
    assert_eq!(status.value["dropped_delivery_count"], 0);
    assert_eq!(status.value["lagged_since_sequence"], json!(null));
}

#[test]
fn out_of_order_fanout_keeps_the_newest_state_and_delivery_cursor_arrival_order() {
    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        policy((8, 8, 2), (8, 8, 2)),
    );
    let listener_id = "External.ConcurrentConsumer".to_string();
    service.handle_listener_control_request(EditorEventListenerControlRequest::Register {
        listener_id: listener_id.clone(),
        display_name: "Concurrent Consumer".to_string(),
    });

    service.record(record(
        4,
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 4.0, y: 0.0 }),
    ));
    service.record(record(
        2,
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 2.0, y: 0.0 }),
    ));
    service.record(record(3, durable_event(3)));
    service.record(record(1, durable_event(1)));

    assert_eq!(
        service
            .journal()
            .records()
            .iter()
            .map(|record| record.sequence.0)
            .collect::<Vec<_>>(),
        vec![4, 3, 1]
    );
    let deliveries = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id: listener_id.clone(),
            after_delivery_cursor: 0,
            max_deliveries: 8,
        },
    );
    assert_eq!(
        deliveries.value["deliveries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|delivery| delivery["sequence"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1, 3, 4]
    );
    let status = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryListenerStatus { listener_id },
    );
    assert_eq!(status.value["coalesced_delivery_count"], 1);
    assert_eq!(status.value["dropped_delivery_count"], 0);
}

#[test]
fn listener_delivery_pages_are_bounded_cursor_ordered_and_continuable() {
    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        policy((8, 8, 8), (8, 8, 8)),
    );
    let listener_id = "External.PagedHistory".to_string();
    service.handle_listener_control_request(EditorEventListenerControlRequest::Register {
        listener_id: listener_id.clone(),
        display_name: "Paged History".to_string(),
    });

    service.record(record(
        4,
        EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 4.0, y: 0.0 }),
    ));
    service.record(record(2, durable_event(2)));
    service.record(record(
        3,
        EditorEvent::Viewport(EditorViewportEvent::Scrolled { delta: 1.0 }),
    ));
    service.record(record(1, durable_event(1)));

    let first = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id: listener_id.clone(),
            after_delivery_cursor: 0,
            max_deliveries: 2,
        },
    );
    assert_eq!(
        first.value["deliveries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|delivery| delivery["sequence"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![4, 2]
    );
    assert_eq!(first.value["next_delivery_cursor"], 2);
    assert_eq!(first.value["has_more"], true);

    let second = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id: listener_id.clone(),
            after_delivery_cursor: 2,
            max_deliveries: 2,
        },
    );
    assert_eq!(
        second.value["deliveries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|delivery| delivery["sequence"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![3, 1]
    );
    assert_eq!(second.value["next_delivery_cursor"], 4);
    assert_eq!(second.value["has_more"], false);

    let empty = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id: listener_id.clone(),
            after_delivery_cursor: 4,
            max_deliveries: 2,
        },
    );
    assert_eq!(empty.value["deliveries"], json!([]));
    assert_eq!(empty.value["next_delivery_cursor"], serde_json::Value::Null);
    assert_eq!(empty.value["has_more"], false);

    let invalid = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id,
            after_delivery_cursor: 0,
            max_deliveries: 0,
        },
    );
    assert_eq!(
        invalid.error.as_deref(),
        Some("editor event listener delivery page size must be between 1 and 256")
    );
}

#[test]
fn delivery_cursor_does_not_skip_a_late_lower_event_sequence() {
    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        policy((8, 8, 8), (8, 8, 8)),
    );
    let listener_id = "External.LateDelivery".to_string();
    service.handle_listener_control_request(EditorEventListenerControlRequest::Register {
        listener_id: listener_id.clone(),
        display_name: "Late Delivery".to_string(),
    });

    service.record(record(2, durable_event(2)));
    let first = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id: listener_id.clone(),
            after_delivery_cursor: 0,
            max_deliveries: 1,
        },
    );
    assert_eq!(first.value["deliveries"][0]["sequence"], 2);
    assert_eq!(first.value["next_delivery_cursor"], 1);

    let acknowledged = service.handle_listener_control_request(
        EditorEventListenerControlRequest::AckDeliveriesThrough {
            listener_id: listener_id.clone(),
            delivery_cursor: 1,
        },
    );
    assert_eq!(acknowledged.value["removed"], 1);

    service.record(record(1, durable_event(1)));
    let late = service.handle_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesPage {
            listener_id,
            after_delivery_cursor: 1,
            max_deliveries: 1,
        },
    );
    assert_eq!(late.value["deliveries"][0]["sequence"], 1);
    assert_eq!(late.value["deliveries"][0]["delivery_cursor"], 2);
    assert_eq!(late.value["has_more"], false);
}

#[test]
fn byte_budget_rejection_is_visible_in_journal_diagnostics() {
    let tiny = budget(8, 32);
    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        EditorEventRetentionPolicy::new(
            EditorEventRetentionBudgets::new(tiny.clone(), tiny.clone(), tiny.clone()),
            EditorEventRetentionBudgets::new(tiny.clone(), tiny.clone(), tiny),
        ),
    );
    service.record(record(
        1,
        EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
    ));

    let journal = service.journal();
    assert!(journal.records().is_empty());
    assert_eq!(
        journal
            .retention_diagnostics()
            .durable_replay
            .dropped_records,
        1
    );
}

#[test]
fn age_budget_expiration_is_visible_in_journal_diagnostics() {
    let short_lived =
        EditorEventRetentionBudget::new(8, 1024 * 1024, Duration::from_millis(1)).unwrap();
    let service = EditorEventService::with_retention_policy(
        SharedEditorMessageBus::default(),
        EditorEventRetentionPolicy::new(
            EditorEventRetentionBudgets::new(
                short_lived,
                budget(8, 1024 * 1024),
                budget(8, 1024 * 1024),
            ),
            EditorEventRetentionBudgets::new(
                budget(8, 1024 * 1024),
                budget(8, 1024 * 1024),
                budget(8, 1024 * 1024),
            ),
        ),
    );
    service.record(record(1, durable_event(1)));
    std::thread::sleep(Duration::from_millis(5));

    let journal = service.journal();
    assert!(journal.records().is_empty());
    assert_eq!(
        journal
            .retention_diagnostics()
            .durable_replay
            .dropped_records,
        1
    );
}

#[test]
fn sequence_state_no_longer_owns_journal_or_listener_fanout() {
    let state = include_str!("../../core/editor_event/service/state.rs");
    assert!(!state.contains("EditorEventJournal"));
    assert!(!state.contains("EditorEventListenerRegistry"));

    let service = include_str!("../../core/editor_event/service/editor_event_service.rs");
    assert!(service.contains("sequence_state: Mutex<EditorEventSequenceState>"));
    assert!(service.contains("journal: Mutex<EditorEventJournalStore>"));
    assert!(service.contains("listeners: Mutex<EditorEventListenerRegistry>"));
    assert!(!service.contains("state: Mutex<EditorEventServiceState>"));
}

#[test]
fn listener_delivery_filter_and_enqueue_stay_outside_the_registry_guard() {
    let service = include_str!("../../core/editor_event/service/editor_event_service.rs");
    let route_snapshot_end = service
        .find("};\n        for route in routes")
        .expect("record must release the registry guard after capturing routes");
    let filter = service
        .find("route.accepts(record.record())")
        .expect("record must filter against immutable listener routes");
    let enqueue = service
        .find("route.enqueue(Arc::clone(&record))")
        .expect("record must enqueue through per-listener route handles");
    assert!(filter > route_snapshot_end);
    assert!(enqueue > route_snapshot_end);
}
