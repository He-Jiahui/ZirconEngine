use crate::core::editor_event::{
    EditorEvent, EditorEventEffect, EditorEventId, EditorEventRecord, EditorEventResult,
    EditorEventSequence, EditorEventSource, EditorEventUndoPolicy, MenuAction,
};

use super::{apply_record_effects, UiHostEventEffects};

#[test]
fn viewport_resize_projection_requires_presentation_and_render_only() {
    let mut effects = UiHostEventEffects::default();
    effects.request_presentation();
    effects.request_render();

    assert!(effects.is_viewport_resize_recompute_compatible());

    effects.request_layout();
    assert!(!effects.is_viewport_resize_recompute_compatible());
}

#[test]
fn viewport_resize_projection_rejects_an_incomplete_effect_set() {
    let mut presentation_only = UiHostEventEffects::default();
    presentation_only.request_presentation();
    assert!(!presentation_only.is_viewport_resize_recompute_compatible());

    let mut render_only = UiHostEventEffects::default();
    render_only.request_render();
    assert!(!render_only.is_viewport_resize_recompute_compatible());
}

#[test]
fn workbench_projection_effect_preserves_coalesced_render_without_global_presentation() {
    let mut effects = UiHostEventEffects::default();
    effects.request_paint_only();
    effects.request_render();
    effects.request_workbench_projection();

    let dirty = effects.dirty_domains();
    assert!(dirty.contains(crate::ui::retained_host::HostInvalidationMask::PAINT_ONLY));
    assert!(dirty.contains(crate::ui::retained_host::HostInvalidationMask::RENDER));
    assert!(dirty.contains(crate::ui::retained_host::HostInvalidationMask::WORKBENCH_PROJECTION));
    assert!(dirty.requires_host_recompute());
    assert!(!dirty.requires_presentation());
}

#[test]
fn stable_shell_content_replaces_generic_layout_invalidation() {
    let mut effects = UiHostEventEffects::default();
    effects.request_layout();
    effects.request_presentation();

    let scope = crate::ui::retained_host::HostShellContentScope::new(
        crate::ui::workbench::layout::ActivityDrawerSlot::LeftBottom,
        crate::ui::workbench::view::ViewInstanceId::new("editor.module_plugins#main"),
    );
    effects.reuse_layout_for_shell_content(scope.clone());

    assert!(!effects.dirty_domains().requires_layout());
    assert!(effects.dirty_domains().requires_presentation());
    assert!(effects
        .dirty_domains()
        .contains(crate::ui::retained_host::HostInvalidationMask::SHELL_CONTENT));
    assert!(!effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(effects.shell_content_scope(), Some(scope));
}

#[test]
fn project_close_effect_requests_retained_close_and_empty_asset_sync() {
    let record = EditorEventRecord {
        event_id: EditorEventId::new(1),
        sequence: EditorEventSequence::new(1),
        source: EditorEventSource::RetainedHost,
        event: EditorEvent::WorkbenchMenu(MenuAction::CloseProject),
        binding_path: None,
        operation_id: None,
        operation_display_name: None,
        operation_arguments: None,
        operation_group: None,
        transaction_id: None,
        save_generation: None,
        effects: vec![
            EditorEventEffect::ProjectCloseRequested,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
        undo_policy: EditorEventUndoPolicy::FutureInverseEvent,
        before_revision: 0,
        after_revision: 0,
        result: EditorEventResult::success(serde_json::json!({
            "revision": 0,
            "changed": false,
        })),
    };
    let mut effects = UiHostEventEffects::default();

    apply_record_effects(&mut effects, &record);

    assert!(effects.close_active_project);
    assert!(effects.sync_asset_workspace);
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
}

#[test]
fn asset_relocation_effect_preserves_background_request_payload() {
    let asset_uuid = "00112233-4455-6677-8899-aabbccddeeff".to_owned();
    let target_locator = "res://environment/cube.zmodel".to_owned();
    let record = EditorEventRecord {
        event_id: EditorEventId::new(2),
        sequence: EditorEventSequence::new(2),
        source: EditorEventSource::RetainedHost,
        event: EditorEvent::Asset(crate::core::editor_event::EditorAssetEvent::RelocateAsset {
            asset_uuid: asset_uuid.clone(),
            target_locator: target_locator.clone(),
        }),
        binding_path: Some("AssetTree/RelocateAsset".to_owned()),
        operation_id: None,
        operation_display_name: None,
        operation_arguments: None,
        operation_group: None,
        transaction_id: None,
        save_generation: None,
        effects: vec![EditorEventEffect::AssetRelocationRequested {
            asset_uuid: asset_uuid.clone(),
            target_locator: target_locator.clone(),
        }],
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: 0,
        after_revision: 0,
        result: EditorEventResult::success(serde_json::json!({ "changed": false })),
    };
    let mut effects = UiHostEventEffects::default();

    apply_record_effects(&mut effects, &record);

    assert_eq!(
        effects.asset_relocation_requested,
        Some(super::AssetRelocationRequest {
            asset_uuid,
            target_locator,
        })
    );
}

#[test]
fn asset_deletion_effect_preserves_background_request_payload() {
    let asset_uuid = "00112233-4455-6677-8899-aabbccddeeff".to_owned();
    let record = EditorEventRecord {
        event_id: EditorEventId::new(3),
        sequence: EditorEventSequence::new(3),
        source: EditorEventSource::RetainedHost,
        event: EditorEvent::Asset(crate::core::editor_event::EditorAssetEvent::DeleteAsset {
            asset_uuid: asset_uuid.clone(),
        }),
        binding_path: Some("AssetContextMenu/DeleteAsset".to_owned()),
        operation_id: None,
        operation_display_name: None,
        operation_arguments: None,
        operation_group: None,
        transaction_id: None,
        save_generation: None,
        effects: vec![EditorEventEffect::AssetDeletionRequested {
            asset_uuid: asset_uuid.clone(),
        }],
        undo_policy: EditorEventUndoPolicy::NonUndoable,
        before_revision: 0,
        after_revision: 0,
        result: EditorEventResult::success(serde_json::json!({ "changed": false })),
    };
    let mut effects = UiHostEventEffects::default();

    apply_record_effects(&mut effects, &record);

    assert_eq!(
        effects.asset_deletion_requested,
        Some(super::AssetDeletionRequest { asset_uuid })
    );
}
