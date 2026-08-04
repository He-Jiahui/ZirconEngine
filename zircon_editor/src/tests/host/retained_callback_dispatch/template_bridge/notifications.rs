use super::super::support::*;
use crate::{
    core::i18n::EditorI18nService,
    core::notifications::{
        DecisionCenterConfig, DecisionNotification, DecisionNotificationCenter, DecisionOption,
        DecisionOptionId, EditorNotificationService, NotificationId, NotificationSource,
        ToastNotification, ToastSeverity,
    },
    ui::activity::{ActivityToastView, activity_toast_views},
    ui::host::play_pending_decision::PlayPendingDecisionOption,
};
use std::time::Duration;

#[test]
fn workbench_toast_queue_and_notification_history_project_core_activity_toasts() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    assert!(bridge.has_control(WORKBENCH_TOAST_CONTROL_ID));
    assert!(bridge.has_control(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID));
    assert_eq!(
        control_string_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "visibility").as_deref(),
        Some("collapsed")
    );

    let i18n = EditorI18nService::default();
    let notifications = EditorNotificationService::default();
    notifications
        .publish_toast(
            ToastNotification::new(
                NotificationId::parse("editor.activity.01.project-save").unwrap(),
                NotificationSource::builtin("editor.activity").unwrap(),
                ToastSeverity::Success,
                "editor.notification.project_saved.title",
                "editor.notification.project_saved.message",
                Duration::from_secs(3),
            )
            .unwrap(),
        )
        .unwrap();
    notifications
        .publish_toast(
            ToastNotification::new(
                NotificationId::parse("editor.activity.02.import-model").unwrap(),
                NotificationSource::builtin("editor.activity").unwrap(),
                ToastSeverity::Info,
                "editor.notification.import_model.title",
                "editor.notification.import_model.message",
                Duration::from_secs(4),
            )
            .unwrap(),
        )
        .unwrap();
    let (now, snapshots) = notifications.live_toast_snapshot();
    let activity_toasts = activity_toast_views(&snapshots, &i18n, now);

    assert!(
        bridge
            .sync_activity_toasts(&activity_toasts)
            .expect("activity toasts should project to workbench overlays")
    );

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
        Some("editor.activity.01.project-save")
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
    assert!(
        toast_queue[1].contains("message=Choose a model file to import into the active project.")
    );

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

    assert!(
        bridge
            .sync_activity_toasts(&[])
            .expect("an empty authority snapshot should clear expired activity toasts")
    );
    assert_eq!(
        control_string_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "visibility").as_deref(),
        Some("collapsed")
    );
    assert!(
        control_string_list_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "toast_queue")
            .is_empty()
    );
}

#[test]
fn pending_play_decision_rows_are_modal_until_a_choice_is_resolved() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
        .expect("decision center should construct");
    let apply = DecisionOptionId::parse("apply").expect("apply option id should be valid");
    let discard = DecisionOptionId::parse("discard").expect("discard option id should be valid");
    let ticket = center
        .publish(
            DecisionNotification::new(
                NotificationId::parse("editor.play.pending_edits.test")
                    .expect("notification id should be valid"),
                NotificationSource::builtin("editor.play")
                    .expect("notification source should be valid"),
                "editor.play.pending_edits.title",
                "editor.play.pending_edits.message",
                vec![
                    DecisionOption::new(apply.clone(), "editor.play.pending_edits.apply")
                        .expect("apply option should construct"),
                    DecisionOption::new(discard, "editor.play.pending_edits.discard")
                        .expect("discard option should construct"),
                ],
            )
            .expect("decision notification should construct"),
        )
        .expect("decision should publish");
    let option = PlayPendingDecisionOption::new(
        "play_pending_decision_test_apply".to_string(),
        ticket,
        apply,
        "Resolve queued play edits".to_string(),
        "Apply one queued edit before the next Play session.".to_string(),
    );

    assert!(
        bridge
            .sync_pending_play_decision_options(std::slice::from_ref(&option))
            .expect("pending decision rows should project")
    );
    let notification_generation = control_int_attribute(
        &bridge,
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "notification_generation",
    );
    assert!(
        !bridge
            .sync_pending_play_decision_options(std::slice::from_ref(&option))
            .expect("same pending decision generation should be a no-op")
    );
    assert_eq!(
        control_int_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "notification_generation"
        ),
        notification_generation
    );
    assert_eq!(
        control_bool_attribute(&bridge, WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, "open"),
        Some(true)
    );
    assert_eq!(
        control_bool_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "close_on_backdrop_click"
        ),
        Some(false)
    );
    assert_eq!(
        control_bool_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "disable_escape_key_down"
        ),
        Some(true)
    );
    assert!(bridge.is_pending_play_decision_option(
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "play_pending_decision_test_apply"
    ));

    assert!(
        bridge
            .sync_pending_play_decision_options(&[])
            .expect("resolved decisions should clear their retained rows")
    );
    assert!(!bridge.is_pending_play_decision_option(
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "play_pending_decision_test_apply"
    ));
    assert_eq!(
        control_bool_attribute(&bridge, WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, "open"),
        Some(false)
    );
    assert_eq!(
        control_bool_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "popup_open"
        ),
        Some(false)
    );
}

#[test]
fn notification_burst_has_bounded_retention_and_explicit_generation_metadata() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let burst = (0..1_000)
        .map(|index| {
            ActivityToastView::new(
                format!("burst-{index}"),
                format!("Burst {index}"),
                "Queued editor update",
                ToastSeverity::Info,
                Duration::from_secs(3),
            )
        })
        .collect::<Vec<_>>();

    assert!(
        bridge
            .sync_activity_toasts(&burst)
            .expect("activity toast burst should project")
    );

    let history = control_string_list_attribute(
        &bridge,
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "notifications",
    );
    let toast_queue =
        control_string_list_attribute(&bridge, WORKBENCH_TOAST_CONTROL_ID, "toast_queue");
    assert_eq!(history.len(), 64);
    assert_eq!(toast_queue.len(), 64);
    assert!(history[0].starts_with("burst-0|"));
    assert!(history[63].starts_with("burst-63|"));
    assert_eq!(
        control_int_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "unread_count"
        ),
        Some(64)
    );
    assert_eq!(
        control_int_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "overflow_count"
        ),
        Some(936)
    );
    assert_eq!(
        control_int_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "notification_generation"
        ),
        Some(1)
    );

    let host_nodes = crate::ui::retained_host::to_host_contract_workbench_window_nodes(Some(
        bridge.host_projection(),
    ));
    let notification_center = (0..host_nodes.row_count())
        .filter_map(|row| host_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID)
        .expect("notification generation should project into the native host");
    assert_eq!(notification_center.notification_generation, 1);
    assert_eq!(notification_center.notification_unread_count, 64);
    assert_eq!(notification_center.notification_overflow_count, 936);
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
