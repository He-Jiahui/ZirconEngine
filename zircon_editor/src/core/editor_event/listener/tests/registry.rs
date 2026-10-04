use std::sync::Arc;

use serde_json::json;

use super::super::super::{
    EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord, EditorEventResult,
    EditorEventSequence, EditorEventSource, EditorEventTransient, EditorEventUndoPolicy,
    SharedEditorEventRecord,
};
use super::EditorEventListenerRegistry;

fn payload(sequence: u64) -> Arc<SharedEditorEventRecord> {
    Arc::new(SharedEditorEventRecord::new(EditorEventRecord {
        event_id: EditorEventId::new(sequence),
        sequence: EditorEventSequence::new(sequence),
        source: EditorEventSource::Headless,
        event: EditorEvent::Transient(EditorEventTransient::OpenCommandPalette),
        binding_path: None,
        operation_id: None,
        operation_display_name: None,
        operation_arguments: None,
        operation_group: None,
        transaction_id: None,
        save_generation: None,
        effects: Vec::<EditorEventEffect>::new(),
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: 0,
        after_revision: 1,
        result: EditorEventResult::success(json!(null)),
    }))
}

#[test]
fn matching_listener_inboxes_share_one_immutable_payload() {
    let mut listeners = EditorEventListenerRegistry::default();
    listeners.register("a", "A").unwrap();
    listeners.register("b", "B").unwrap();
    let payload = payload(1);

    for route in listeners.delivery_routes().iter() {
        if route.accepts(payload.record()) {
            route.enqueue(Arc::clone(&payload));
        }
    }

    assert_eq!(Arc::strong_count(&payload), 3);
}

#[test]
fn delivery_routes_are_rebuilt_when_listener_configuration_changes() {
    let mut listeners = EditorEventListenerRegistry::default();
    listeners.register("a", "A").unwrap();
    assert_eq!(listeners.delivery_routes().len(), 1);

    listeners.set_enabled("a", false).unwrap();
    assert!(listeners.delivery_routes().is_empty());

    listeners.set_enabled("a", true).unwrap();
    listeners
        .set_filter(
            "a",
            super::super::EditorEventListenerFilter::default().failures_only(),
        )
        .unwrap();
    assert_eq!(listeners.delivery_routes().len(), 1);
}

#[test]
fn in_flight_route_snapshot_uses_its_captured_filter_and_new_snapshots_use_reconfiguration() {
    let mut listeners = EditorEventListenerRegistry::default();
    listeners.register("a", "A").unwrap();
    let in_flight_routes = listeners.delivery_routes();
    let event = payload(2);

    listeners
        .set_filter(
            "a",
            super::super::EditorEventListenerFilter::default().failures_only(),
        )
        .unwrap();
    let filtered_routes = listeners.delivery_routes();
    assert!(in_flight_routes[0].accepts(event.record()));
    assert!(!filtered_routes[0].accepts(event.record()));

    in_flight_routes[0].enqueue(Arc::clone(&event));
    for route in filtered_routes.iter() {
        if route.accepts(event.record()) {
            route.enqueue(Arc::clone(&event));
        }
    }
    assert_eq!(
        listeners
            .listener_handle("a")
            .unwrap()
            .status()
            .pending_delivery_count,
        1
    );

    listeners.set_enabled("a", false).unwrap();
    assert!(listeners.delivery_routes().is_empty());
    let detached_handle = listeners.listener_handle("a").unwrap();
    listeners.unregister("a").unwrap();
    assert!(listeners.delivery_routes().is_empty());
    assert!(listeners.listener_handle("a").is_err());

    // The snapshot acquired before unregister remains a valid in-flight owner.
    in_flight_routes[0].enqueue(event);
    assert_eq!(detached_handle.status().pending_delivery_count, 2);
}
