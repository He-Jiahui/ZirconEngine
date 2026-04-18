use crate::core::editor_event::{
    EditorAssetEvent, EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord,
    EditorEventResult, EditorEventSequence, EditorEventSource, EditorEventUndoPolicy,
};
use crate::ui::slint_host::event_bridge::{apply_record_effects, SlintDispatchEffects};

fn record_with_event_and_effects(
    event: EditorEvent,
    effects: Vec<EditorEventEffect>,
) -> EditorEventRecord {
    EditorEventRecord {
        event_id: EditorEventId::new(1),
        sequence: EditorEventSequence::new(1),
        source: EditorEventSource::Slint,
        event,
        effects,
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: 0,
        after_revision: 1,
        result: EditorEventResult::success(serde_json::json!({
            "revision": 1,
            "changed": true,
        })),
    }
}

fn record_with_effects(effects: Vec<EditorEventEffect>) -> EditorEventRecord {
    record_with_event_and_effects(
        EditorEvent::Asset(EditorAssetEvent::SetSearchQuery {
            query: "cube".to_string(),
        }),
        effects,
    )
}

#[test]
fn record_effects_mark_render_layout_and_asset_refresh_flags_for_slint_host() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::ProjectOpenRequested,
            EditorEventEffect::LayoutChanged,
            EditorEventEffect::RenderChanged,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.presentation_dirty);
    assert!(effects.layout_dirty);
    assert!(effects.render_dirty);
    assert!(effects.sync_asset_workspace);
    assert!(effects.reset_active_layout_preset);
}

#[test]
fn present_welcome_effect_only_marks_welcome_presentation_path() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::PresentWelcomeRequested,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.present_welcome_surface);
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
    assert!(!effects.reset_active_layout_preset);
}

#[test]
fn asset_preview_refresh_effect_is_local_to_presentation_without_backend_sync() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::AssetPreviewRefreshRequested,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.presentation_dirty);
    assert!(!effects.sync_asset_workspace);
    assert!(!effects.refresh_asset_details);
    assert!(effects.refresh_visible_asset_previews);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
}

#[test]
fn asset_details_refresh_effect_does_not_require_backend_sync() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::AssetDetailsRefreshRequested,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.presentation_dirty);
    assert!(!effects.sync_asset_workspace);
    assert!(effects.refresh_asset_details);
    assert!(!effects.refresh_visible_asset_previews);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
}

#[test]
fn layout_preset_events_project_active_preset_name_into_slint_effects() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::Layout(crate::LayoutCommand::SavePreset {
                name: "rider".to_string(),
            }),
            vec![
                EditorEventEffect::LayoutChanged,
                EditorEventEffect::PresentationChanged,
            ],
        ),
    );

    assert_eq!(effects.active_layout_preset_name.as_deref(), Some("rider"));
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
}

#[test]
fn reset_layout_event_clears_active_preset_selection_in_host_effects() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::Layout(crate::LayoutCommand::ResetToDefault),
            vec![
                EditorEventEffect::LayoutChanged,
                EditorEventEffect::PresentationChanged,
            ],
        ),
    );

    assert!(effects.reset_active_layout_preset);
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(effects.active_layout_preset_name, None);
}

#[test]
fn menu_reset_layout_event_also_clears_active_preset_selection_in_host_effects() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::WorkbenchMenu(crate::MenuAction::ResetLayout),
            vec![
                EditorEventEffect::LayoutChanged,
                EditorEventEffect::PresentationChanged,
            ],
        ),
    );

    assert!(effects.reset_active_layout_preset);
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(effects.active_layout_preset_name, None);
}
