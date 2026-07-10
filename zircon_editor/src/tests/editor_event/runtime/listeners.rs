use super::*;

#[test]
fn event_listener_control_gates_named_event_deliveries() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_control");
    let listener_id = "External.HistoryPanel".to_string();

    let registered = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "History Panel".to_string(),
        },
    );
    assert!(registered.error.is_none());

    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetEnabled {
            listener_id: listener_id.clone(),
            enabled: false,
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();
    let disabled_deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries {
            listener_id: listener_id.clone(),
        },
    );
    assert_eq!(
        disabled_deliveries.value["deliveries"]
            .as_array()
            .expect("deliveries")
            .len(),
        0
    );

    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetEnabled {
            listener_id: listener_id.clone(),
            enabled: true,
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    assert_eq!(
        deliveries.value["deliveries"][0]["operation_id"],
        "scene.node.create_cube"
    );
    assert_eq!(deliveries.value["deliveries"][0]["sequence"], 2);
}

#[test]
fn event_listener_filter_limits_delivery_by_operation_path_prefix() {
    use crate::core::editor_event::{EditorEventListenerControlRequest, EditorEventListenerFilter};
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_filter");
    let listener_id = "External.SceneHistory".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Scene History".to_string(),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetFilter {
            listener_id: listener_id.clone(),
            filter: EditorEventListenerFilter::operation_prefix("Scene.Node."),
        },
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("window.layout.reset").unwrap(),
            ),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["operation_id"], "scene.node.create_cube");
}

#[test]
fn event_listener_filter_limits_delivery_by_operation_group() {
    use crate::core::editor_event::{EditorEventListenerControlRequest, EditorEventListenerFilter};
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_group_filter");
    let listener_id = "External.TransformDrag".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Transform Drag".to_string(),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetFilter {
            listener_id: listener_id.clone(),
            filter: EditorEventListenerFilter::operation_group("Viewport.TransformDrag.42"),
        },
    );

    let operation_path = EditorOperationPath::parse("scene.node.create_cube").unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation_path.clone())
                .with_operation_group("Viewport.TransformDrag.41"),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation_path.clone())
                .with_operation_group("Viewport.TransformDrag.42"),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::UiBinding,
            EditorOperationInvocation::new(operation_path),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(
        deliveries[0]["operation_group"],
        json!("Viewport.TransformDrag.42")
    );
}

#[test]
fn event_listener_filter_limits_delivery_by_source_and_failure_state() {
    use crate::core::editor_event::{
        EditorEventListenerControlRequest, EditorEventListenerFilter, EditorEventSource,
    };
    use crate::core::editor_operation::{
        EditorOperationControlRequest, EditorOperationInvocation, EditorOperationPath,
        EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_source_filter");
    let listener_id = "External.CliFailures".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "CLI Failure Monitor".to_string(),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetFilter {
            listener_id: listener_id.clone(),
            filter: EditorEventListenerFilter::source(EditorEventSource::Cli).failures_only(),
        },
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();
    let cli_success = runtime
        .runtime
        .handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            )),
        );
    assert!(cli_success.error.is_none());
    let cli_failure = runtime
        .runtime
        .handle_operation_control_request_from_source(
            EditorOperationSource::Cli,
            EditorOperationControlRequest::InvokeOperation(EditorOperationInvocation::new(
                EditorOperationPath::parse("weather.missing.action").unwrap(),
            )),
        );
    assert!(cli_failure.error.is_some());

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["source"], "Cli");
    assert_eq!(deliveries[0]["operation_id"], "weather.missing.action");
    let result_error = deliveries[0]["result"]["error"]
        .as_str()
        .expect("result error");
    assert!(result_error.contains("weather.missing.action"));
    assert!(result_error.contains("is not registered"));
}

#[test]
fn event_listener_control_clears_operation_path_filter() {
    use crate::core::editor_event::{EditorEventListenerControlRequest, EditorEventListenerFilter};
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_clear_filter");
    let listener_id = "External.DynamicPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Dynamic Panel".to_string(),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::SetFilter {
            listener_id: listener_id.clone(),
            filter: EditorEventListenerFilter::operation_prefix("Scene.Node."),
        },
    );
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::ClearFilter {
            listener_id: listener_id.clone(),
        },
    );

    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("window.layout.reset").unwrap(),
            ),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["operation_id"], "window.layout.reset");
}

#[test]
fn event_listener_control_unregisters_listener_and_drops_deliveries() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_unregister");
    let listener_id = "External.TemporaryPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Temporary Panel".to_string(),
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();

    let unregistered = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Unregister {
            listener_id: listener_id.clone(),
        },
    );
    assert!(unregistered.error.is_none());

    let listeners = runtime
        .runtime
        .handle_event_listener_control_request(EditorEventListenerControlRequest::ListListeners);
    assert!(listeners.value["listeners"]
        .as_array()
        .expect("listeners")
        .is_empty());

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    assert_eq!(
        deliveries.error.as_deref(),
        Some("editor event listener External.TemporaryPanel is not registered")
    );
}

#[test]
fn event_listener_control_rejects_unknown_listener_queries() {
    use crate::core::editor_event::EditorEventListenerControlRequest;

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_unknown_query");
    let listener_id = "External.MissingPanel".to_string();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries {
            listener_id: listener_id.clone(),
        },
    );
    assert_eq!(
        deliveries.error.as_deref(),
        Some("editor event listener External.MissingPanel is not registered")
    );

    let cursor = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesSince {
            listener_id: listener_id.clone(),
            after_sequence: 10,
        },
    );
    assert_eq!(
        cursor.error.as_deref(),
        Some("editor event listener External.MissingPanel is not registered")
    );

    let ack = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::AckDeliveriesThrough {
            listener_id,
            sequence: 10,
        },
    );
    assert_eq!(
        ack.error.as_deref(),
        Some("editor event listener External.MissingPanel is not registered")
    );
}

#[test]
fn event_listener_control_queries_deliveries_after_sequence_cursor() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_cursor");
    let listener_id = "External.PollingPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Polling Panel".to_string(),
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveriesSince {
            listener_id,
            after_sequence: 1,
        },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["sequence"], 2);
    assert_eq!(deliveries[0]["operation_id"], "scene.node.create_cube");
}

#[test]
fn event_listener_control_acknowledges_deliveries_through_sequence() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_ack");
    let listener_id = "External.StreamingPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Streaming Panel".to_string(),
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();

    let ack = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::AckDeliveriesThrough {
            listener_id: listener_id.clone(),
            sequence: 1,
        },
    );
    assert!(ack.error.is_none());
    assert_eq!(ack.value["removed"], 1);

    let deliveries = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryDeliveries { listener_id },
    );
    let deliveries = deliveries.value["deliveries"]
        .as_array()
        .expect("deliveries");
    assert_eq!(deliveries.len(), 1);
    assert_eq!(deliveries[0]["sequence"], 2);
}

#[test]
fn event_listener_control_reports_listener_status_with_pending_delivery_bounds() {
    use crate::core::editor_event::EditorEventListenerControlRequest;
    use crate::core::editor_operation::{
        EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
    };

    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_editor_event_listener_status");
    let listener_id = "External.StatusPanel".to_string();
    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::Register {
            listener_id: listener_id.clone(),
            display_name: "Status Panel".to_string(),
        },
    );
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();
    runtime
        .runtime
        .invoke_operation(
            EditorOperationSource::Remote,
            EditorOperationInvocation::new(
                EditorOperationPath::parse("scene.node.create_cube").unwrap(),
            ),
        )
        .unwrap();

    let status = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryListenerStatus {
            listener_id: listener_id.clone(),
        },
    );
    assert!(status.error.is_none());
    assert_eq!(status.value["listener_id"], listener_id);
    assert_eq!(status.value["descriptor"]["display_name"], "Status Panel");
    assert_eq!(status.value["descriptor"]["enabled"], true);
    assert_eq!(status.value["pending_delivery_count"], 2);
    assert_eq!(status.value["first_pending_sequence"], 1);
    assert_eq!(status.value["last_pending_sequence"], 2);

    runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::AckDeliveriesThrough {
            listener_id: listener_id.clone(),
            sequence: 2,
        },
    );
    let empty_status = runtime.runtime.handle_event_listener_control_request(
        EditorEventListenerControlRequest::QueryListenerStatus { listener_id },
    );
    assert_eq!(empty_status.value["pending_delivery_count"], 0);
    assert_eq!(empty_status.value["first_pending_sequence"], json!(null));
    assert_eq!(empty_status.value["last_pending_sequence"], json!(null));
}
