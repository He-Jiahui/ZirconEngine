use super::super::support::*;
use crate::ui::retained_host::workbench_notifications::{
    WorkbenchNotification, WorkbenchNotificationSeverity,
};

#[test]
fn workbench_toast_queue_and_notification_history_receive_editor_notifications() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    assert!(bridge.has_control(WORKBENCH_TOAST_CONTROL_ID));
    assert!(bridge.has_control(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID));
    assert_eq!(
        control_string_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "visibility").as_deref(),
        Some("collapsed")
    );

    let project_saved = WorkbenchNotification::new(
        "event-1-save",
        "Project saved",
        "Project state was written to disk.",
        WorkbenchNotificationSeverity::Success,
    );
    let import_requested = WorkbenchNotification::new(
        "event-1-import",
        "Import model",
        "Choose a model file to import into the active project.",
        WorkbenchNotificationSeverity::Info,
    )
    .with_action_label("Import")
    .with_duration_ms(4_000);

    assert!(bridge
        .push_workbench_notifications(&[project_saved, import_requested])
        .expect("notifications should publish to workbench overlays"));

    assert_eq!(
        control_string_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "visibility").as_deref(),
        Some("visible")
    );
    assert_eq!(
        control_bool_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "popup_open"),
        Some(true)
    );
    assert_eq!(
        control_string_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "current_toast_id")
            .as_deref(),
        Some("event-1-save")
    );
    assert_eq!(
        control_string_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "text").as_deref(),
        Some("Project state was written to disk.")
    );
    assert_eq!(
        control_string_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "severity").as_deref(),
        Some("success")
    );
    assert_eq!(
        control_int_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "queue_length"),
        Some(2)
    );
    let toast_queue =
        control_string_list_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "toast_queue");
    assert_eq!(toast_queue.len(), 2);
    assert!(toast_queue[0].contains("message=Project state was written to disk."));
    assert!(toast_queue[1].contains("action_label=Import"));

    assert_eq!(
        control_string_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "visibility"
        )
        .as_deref(),
        Some("visible")
    );
    assert!(
        !control_bool_attribute(&bridge, WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, "open")
            .unwrap_or(false)
    );
    assert_eq!(
        control_bool_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "input_interactive"
        ),
        Some(false)
    );
    assert_eq!(
        control_int_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "unread_count"
        ),
        Some(2)
    );
    let history = control_string_list_attribute(
        &bridge,
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "notifications",
    );
    assert_eq!(history.len(), 2);
    assert!(history[0].contains("title=Project saved"));
    assert!(history[0].contains("severity=success"));
    assert!(history[1].contains("title=Import model"));

    let host_nodes = crate::ui::retained_host::to_host_contract_workbench_window_nodes(Some(
        bridge.host_projection(),
    ));
    let notification_center = (0..host_nodes.row_count())
        .filter_map(|row| host_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID)
        .expect("notification center should project history for the native host");
    assert!(!notification_center.popup_open);
    assert_eq!(notification_center.structured_options.row_count(), 2);
    assert_eq!(
        notification_center
            .structured_options
            .row_data(0)
            .expect("first notification should project")
            .label
            .as_str(),
        "Project saved"
    );
}

fn control_string_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<String> {
    control_attribute(bridge, control_id, property)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn control_bool_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<bool> {
    control_attribute(bridge, control_id, property).and_then(toml::Value::as_bool)
}

fn control_int_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<i64> {
    control_attribute(bridge, control_id, property).and_then(toml::Value::as_integer)
}

fn control_string_list_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Vec<String> {
    control_attribute(bridge, control_id, property)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}

fn control_attribute<'a>(
    bridge: &'a BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Option<&'a toml::Value> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
            .and_then(|metadata| metadata.attributes.get(property))
    })
}
