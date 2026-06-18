use crate::core::editor_event::{EditorAssetEvent, EditorEvent, EditorEventEffect, MenuAction};
use crate::ui::retained_host::event_bridge::{apply_record_effects, UiHostEventEffects};
use crate::ui::retained_host::workbench_notifications::WorkbenchNotificationSeverity;
use crate::ui::retained_host::HostInvalidationMask;

use super::support::record_with_event_and_effects;

#[test]
fn save_and_import_records_queue_workbench_notifications_for_retained_host() {
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::WorkbenchMenu(MenuAction::SaveProject),
            vec![EditorEventEffect::ProjectSaveRequested],
        ),
    );

    assert!(effects.presentation_dirty);
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PRESENTATION_DATA));
    assert_eq!(effects.workbench_notifications.len(), 1);
    let save_notification = &effects.workbench_notifications[0];
    assert_eq!(save_notification.id, "editor-event-1-project-save");
    assert_eq!(save_notification.title, "Project saved");
    assert_eq!(
        save_notification.severity,
        WorkbenchNotificationSeverity::Success
    );

    let mut effects = UiHostEventEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::Asset(EditorAssetEvent::ImportModel),
            vec![EditorEventEffect::ImportModelRequested],
        ),
    );

    assert!(effects.import_model_requested);
    assert!(effects.presentation_dirty);
    assert_eq!(effects.workbench_notifications.len(), 1);
    let import_notification = &effects.workbench_notifications[0];
    assert_eq!(import_notification.id, "editor-event-1-import-model");
    assert_eq!(import_notification.title, "Import model");
    assert_eq!(
        import_notification.severity,
        WorkbenchNotificationSeverity::Info
    );
    assert_eq!(import_notification.action_label.as_deref(), Some("Import"));
}
