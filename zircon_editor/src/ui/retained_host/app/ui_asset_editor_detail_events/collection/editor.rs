use super::{CollectionEventDispatch, EditorManager, ViewInstanceId};

pub(super) fn dispatch_editor_collection_event(
    editor_manager: &EditorManager,
    instance_id: &ViewInstanceId,
    collection_id: &str,
    event_kind: &str,
    item_index: usize,
) -> CollectionEventDispatch {
    match (collection_id, event_kind) {
        ("matched_style_rule", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_matched_style_rule(instance_id, item_index)
                .map(|_| ()),
        ),
        ("palette", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_palette_index(instance_id, item_index)
                .map(|_| ()),
        ),
        ("palette_target_candidate", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_palette_target_candidate(instance_id, item_index)
                .map(|_| ()),
        ),
        ("hierarchy", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_hierarchy_index(instance_id, item_index)
                .map(|_| ()),
        ),
        ("hierarchy", "activated") => Some(
            editor_manager
                .activate_ui_asset_editor_hierarchy_index(instance_id, item_index)
                .map(|_| ()),
        ),
        ("preview", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_preview_index(instance_id, item_index)
                .map(|_| ()),
        ),
        ("preview", "activated") => Some(
            editor_manager
                .activate_ui_asset_editor_preview_index(instance_id, item_index)
                .map(|_| ()),
        ),
        ("source_outline", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_source_outline_index(instance_id, item_index)
                .map(|_| ()),
        ),
        ("preview_mock_subject", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_preview_mock_subject(instance_id, item_index)
                .map(|_| ()),
        ),
        ("preview_mock", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_preview_mock_property(instance_id, item_index)
                .map(|_| ()),
        ),
        ("preview_mock_nested", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_preview_mock_nested_entry(instance_id, item_index)
                .map(|_| ()),
        ),
        _ => None,
    }
}
