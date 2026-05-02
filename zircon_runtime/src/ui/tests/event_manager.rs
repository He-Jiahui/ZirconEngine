use serde_json::json;

use crate::ui::event_ui::UiEventManager;
use zircon_runtime_interface::ui::{
    binding::{UiBindingCall, UiBindingValue, UiEventBinding, UiEventKind, UiEventPath},
    event_ui::{
        UiActionDescriptor, UiControlResponse, UiInvocationError, UiInvocationRequest,
        UiNodeDescriptor, UiNodeId, UiNodePath, UiNotification, UiParameterDescriptor,
        UiPropertyDescriptor, UiReflectionSnapshot, UiStateFlags, UiTreeId, UiValueType,
    },
};

#[test]
fn ui_event_manager_invokes_route_and_binding_through_same_handler() {
    let mut manager = UiEventManager::default();
    let binding = UiEventBinding::new(
        UiEventPath::new(
            "AnimationClipEditorView",
            "AddFrameButton",
            UiEventKind::Click,
        ),
        UiBindingCall::new("PositionOfTrackAndFrame")
            .with_argument(UiBindingValue::string("root/child:transform"))
            .with_argument(UiBindingValue::unsigned(24)),
    );
    let route_id = manager.register_route(binding.clone(), |_context| {
        Ok(json!({ "kind": "add_frame", "frame": 24 }))
    });
    let (_subscription_id, receiver) = manager.subscribe();

    let direct = manager.invoke_route(route_id, Vec::new());
    let through_binding = manager.invoke_binding(binding);

    assert_eq!(
        direct.value,
        Some(json!({ "kind": "add_frame", "frame": 24 }))
    );
    assert_eq!(through_binding.value, direct.value);
    assert_eq!(direct.route_id, Some(route_id));
    let notification = receiver.recv().unwrap();
    assert!(matches!(
        notification,
        UiNotification::Invocation(result) if result.route_id == Some(route_id)
    ));
}

#[test]
fn ui_event_manager_tracks_reflection_diff_and_control_requests() {
    let mut manager = UiEventManager::default();
    let refresh_binding = UiEventBinding::new(
        UiEventPath::new("InspectorView", "RefreshButton", UiEventKind::Click),
        UiBindingCall::new("RefreshInspector"),
    );
    let refresh_route =
        manager.register_route(refresh_binding, |_context| Ok(json!({ "refreshed": true })));
    let snapshot = UiReflectionSnapshot::new(
        UiTreeId::new("editor.workbench"),
        vec![UiNodeId::new(1)],
        vec![UiNodeDescriptor::new(
            UiNodeId::new(1),
            UiNodePath::new("editor/workbench/inspector"),
            "InspectorView",
            "Inspector",
        )
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: true,
            clickable: false,
            hoverable: false,
            focusable: true,
            pressed: false,
            checked: false,
            dirty: false,
        })
        .with_property(
            UiPropertyDescriptor::new("title", UiValueType::String, json!("Inspector"))
                .writable(true),
        )
        .with_action(
            UiActionDescriptor::new("refresh", UiEventKind::Click, "RefreshInspector")
                .with_callable_from_remote(true)
                .with_route_id(refresh_route)
                .with_parameter(UiParameterDescriptor::new("force", UiValueType::Bool)),
        )],
    );
    let (_subscription_id, receiver) = manager.subscribe();

    manager.replace_tree(snapshot);
    let response = manager.handle_request(UiInvocationRequest::SetProperty {
        node_path: UiNodePath::new("editor/workbench/inspector"),
        property_name: "title".to_string(),
        value: json!("Inspector Draft"),
    });

    assert!(matches!(response, UiControlResponse::Ack));
    let property = manager.handle_request(UiInvocationRequest::QueryProperty {
        node_path: UiNodePath::new("editor/workbench/inspector"),
        property_name: "title".to_string(),
    });
    assert!(matches!(
        property,
        UiControlResponse::Property(Some(property))
            if property.reflected_value == json!("Inspector Draft")
    ));
    let action = manager.handle_request(UiInvocationRequest::CallAction {
        node_path: UiNodePath::new("editor/workbench/inspector"),
        action_id: "refresh".to_string(),
        arguments: vec![UiBindingValue::Bool(true)],
    });
    assert!(matches!(
        action,
        UiControlResponse::Invocation(result)
            if result.route_id == Some(refresh_route)
                && result.value == Some(json!({ "refreshed": true }))
    ));

    let notification = receiver.recv().unwrap();
    assert!(matches!(
        notification,
        UiNotification::ReflectionDiff(diff)
            if diff.changed_nodes.contains(&UiNodeId::new(1))
    ));
}

#[test]
fn ui_event_manager_reports_explicit_error_for_callable_action_without_route() {
    let mut manager = UiEventManager::default();
    manager.replace_tree(UiReflectionSnapshot::new(
        UiTreeId::new("editor.workbench"),
        vec![UiNodeId::new(1)],
        vec![UiNodeDescriptor::new(
            UiNodeId::new(1),
            UiNodePath::new("editor/workbench/inspector"),
            "InspectorView",
            "Inspector",
        )
        .with_action(
            UiActionDescriptor::new("refresh", UiEventKind::Click, "RefreshInspector")
                .with_callable_from_remote(true),
        )],
    ));

    let response = manager.handle_request(UiInvocationRequest::CallAction {
        node_path: UiNodePath::new("editor/workbench/inspector"),
        action_id: "refresh".to_string(),
        arguments: Vec::new(),
    });

    assert!(matches!(
        response,
        UiControlResponse::Invocation(result)
            if result.error
                == Some(UiInvocationError::ActionMissingRoute {
                    node_path: "editor/workbench/inspector".to_string(),
                    action_id: "refresh".to_string(),
                })
    ));
}
