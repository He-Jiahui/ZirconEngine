use super::super::support::*;
use crate::{
    core::notifications::{
        DecisionCenterConfig, DecisionNotification, DecisionNotificationCenter, DecisionOption,
        DecisionOptionId, EditorNotificationService, NotificationId, NotificationSource,
        ToastNotification, ToastSeverity,
    },
    core::{i18n::EditorI18nService, jobs::JobId},
    ui::activity::{
        ActivityProgressView, ActivityToastView, activity_decision_options, activity_toast_views,
    },
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
            .sync_notification_snapshot(&[], &activity_toasts, &[])
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
            .sync_notification_snapshot(&[], &[], &[])
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
    assert!(
        control_string_list_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "notifications",
        )
        .is_empty(),
        "the retained bridge must not preserve a second toast history after the core snapshot clears"
    );
}

#[test]
fn pending_decision_rows_are_modal_until_a_choice_is_resolved() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
        .expect("decision center should construct");
    let apply = DecisionOptionId::parse("apply").expect("apply option id should be valid");
    let discard = DecisionOptionId::parse("discard").expect("discard option id should be valid");
    center
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
    let options =
        activity_decision_options(&center.pending_snapshot(), &EditorI18nService::default());

    assert!(
        bridge
            .sync_notification_snapshot(&options, &[], &[])
            .expect("pending decision rows should project")
    );
    let notification_generation = control_int_attribute(
        &bridge,
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "notification_generation",
    );
    assert!(
        !bridge
            .sync_notification_snapshot(&options, &[], &[])
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
    assert!(bridge.is_pending_activity_decision_option(
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "editor.play.pending_edits.test:apply"
    ));

    assert!(
        bridge
            .sync_notification_snapshot(&[], &[], &[])
            .expect("resolved decisions should clear their retained rows")
    );
    assert!(!bridge.is_pending_activity_decision_option(
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "editor.play.pending_edits.test:apply"
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
fn empty_current_snapshot_preserves_current_activity_toast_selection() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let toast = ActivityToastView::new(
        "editor.activity.pending-sync",
        "Project saved",
        "Project state was written to disk.",
        ToastSeverity::Success,
        Duration::from_secs(3),
    );

    assert!(
        bridge
            .sync_notification_snapshot(&[], std::slice::from_ref(&toast), &[])
            .expect("activity toast should project")
    );
    assert!(
        !bridge
            .sync_notification_snapshot(&[], std::slice::from_ref(&toast), &[])
            .expect("the same current core snapshot should be a no-op")
    );
    assert_eq!(
        control_string_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "selected_notification_id"
        )
        .as_deref(),
        Some("editor.activity.pending-sync")
    );
}

#[test]
fn toast_countdown_does_not_rebuild_the_workbench_projection() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let initial = ActivityToastView::new(
        "editor.activity.stable-countdown",
        "Project saved",
        "Project state was written to disk.",
        ToastSeverity::Success,
        Duration::from_secs(3),
    );
    let countdown = ActivityToastView::new(
        "editor.activity.stable-countdown",
        "Project saved",
        "Project state was written to disk.",
        ToastSeverity::Success,
        Duration::from_secs(2),
    );

    assert!(
        bridge
            .sync_notification_snapshot(&[], std::slice::from_ref(&initial), &[])
            .expect("initial toast should project")
    );
    let projection_after_initial = bridge.host_projection().clone();

    assert!(
        !bridge
            .sync_notification_snapshot(&[], std::slice::from_ref(&countdown), &[])
            .expect("a countdown-only update should be a no-op")
    );
    assert_eq!(bridge.host_projection(), &projection_after_initial);
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
            .sync_notification_snapshot(&[], &burst, &[])
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

    let retained = burst.iter().take(64).cloned().collect::<Vec<_>>();
    assert!(
        bridge
            .sync_notification_snapshot(&[], &retained, &[])
            .expect("overflow metadata should refresh when retained history is unchanged")
    );
    assert_eq!(
        control_int_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "overflow_count"
        ),
        Some(0)
    );
}

#[test]
fn active_progress_rows_are_projected_and_removed_when_the_core_snapshot_is_empty() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let progress = ActivityProgressView::new(
        "editor.activity.import-progress",
        JobId::new(17),
        "Import complete",
        "Converting terrain materials",
        Some(75),
    );

    assert!(
        bridge
            .sync_notification_snapshot(&[], &[], std::slice::from_ref(&progress))
            .expect("active core progress should project into notification history")
    );
    let history = control_string_list_attribute(
        &bridge,
        WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
        "notifications",
    );
    assert_eq!(history.len(), 1);
    assert!(history[0].contains("kind=progress"));
    assert!(history[0].contains("job_id=17"));
    assert!(history[0].contains("percent=75"));
    assert!(history[0].contains("message=Converting terrain materials"));
    assert!(
        !bridge
            .sync_notification_snapshot(&[], &[], std::slice::from_ref(&progress))
            .expect("the same active progress snapshot should not trigger a refresh")
    );

    assert!(
        bridge
            .sync_notification_snapshot(&[], &[], &[])
            .expect("stale progress rows should be removed when the core snapshot clears")
    );
    assert!(
        control_string_list_attribute(
            &bridge,
            WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID,
            "notifications",
        )
        .is_empty()
    );
}

#[test]
fn notification_center_uses_the_live_toolbar_control_anchor() {
    let _guard = env_lock().lock().unwrap();
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let metadata = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata.as_ref().filter(|metadata| {
                metadata.control_id.as_deref() == Some(WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID)
            })
        })
        .expect("notification center metadata should exist");

    assert_eq!(
        metadata.widget.popup_anchor.control_id(),
        Some("WorkbenchWindowTopToolbarRegion")
    );
    assert_eq!(metadata.widget.open_property.as_deref(), Some("popup_open"));
    assert_eq!(
        metadata.widget.resolved_behavior(&metadata.component),
        zircon_runtime_interface::ui::widget::UiWidgetBehavior::Popup
    );
    for property in [
        "popup_anchor_x",
        "popup_anchor_y",
        "popup_anchor_width",
        "popup_anchor_height",
    ] {
        assert!(!metadata.attributes.contains_key(property), "{property}");
    }
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
