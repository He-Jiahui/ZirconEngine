use crate::editor_event::{
    EditorAssetEvent, EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord,
    EditorEventResult, EditorEventSequence, EditorEventSource, EditorEventUndoPolicy,
};
use crate::host::slint_host::event_bridge::{apply_record_effects, SlintDispatchEffects};

fn record_with_effects(effects: Vec<EditorEventEffect>) -> EditorEventRecord {
    EditorEventRecord {
        event_id: EditorEventId::new(1),
        sequence: EditorEventSequence::new(1),
        source: EditorEventSource::Slint,
        event: EditorEvent::Asset(EditorAssetEvent::SetSearchQuery {
            query: "cube".to_string(),
        }),
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
fn asset_workspace_effect_requests_preview_refresh_without_layout_or_render_dirtiness() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::AssetWorkspaceChanged,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.presentation_dirty);
    assert!(effects.sync_asset_workspace);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
}
