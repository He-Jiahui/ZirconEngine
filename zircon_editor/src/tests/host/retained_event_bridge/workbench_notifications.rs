use crate::core::editor_event::{EditorAssetEvent, EditorEvent, EditorEventEffect, MenuAction};
use crate::core::notifications::ToastSeverity;
use crate::ui::retained_host::HostInvalidationMask;
use crate::ui::retained_host::event_bridge::{UiHostEventEffects, apply_record_effects};

use super::support::record_with_event_and_effects;

#[test]
fn save_and_import_records_queue_typed_toasts_for_retained_host() {
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::WorkbenchMenu(MenuAction::SaveProject),
            vec![EditorEventEffect::ProjectSaveRequested],
        ),
    );

    assert!(effects.presentation_dirty);
    assert!(
        effects
            .dirty_domains()
            .contains(HostInvalidationMask::PRESENTATION_DATA)
    );
    assert_eq!(effects.toast_notifications.len(), 1);
    let save_notification = &effects.toast_notifications[0];
    assert_eq!(
        save_notification.id().as_str(),
        "editor.event.1.project-save"
    );
    assert_eq!(
        save_notification.title_key(),
        "editor.notification.project_saved.title"
    );
    assert_eq!(save_notification.severity(), ToastSeverity::Success);

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
    assert_eq!(effects.toast_notifications.len(), 1);
    let import_notification = &effects.toast_notifications[0];
    assert_eq!(
        import_notification.id().as_str(),
        "editor.event.1.import-model"
    );
    assert_eq!(
        import_notification.title_key(),
        "editor.notification.import_model.title"
    );
    assert_eq!(import_notification.severity(), ToastSeverity::Info);
}
