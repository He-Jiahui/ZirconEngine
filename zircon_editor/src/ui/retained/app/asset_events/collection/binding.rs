use super::{CollectionEventDispatch, EditorManager, ViewInstanceId};

pub(super) fn dispatch_binding_collection_event(
    editor_manager: &EditorManager,
    instance_id: &ViewInstanceId,
    collection_id: &str,
    event_kind: &str,
    item_index: usize,
) -> CollectionEventDispatch {
    match (collection_id, event_kind) {
        ("binding", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_binding(instance_id, item_index)
                .map(|_| ()),
        ),
        ("binding_event", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_binding_event_option(instance_id, item_index)
                .map(|_| ()),
        ),
        ("binding_action_kind", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_binding_action_kind(instance_id, item_index)
                .map(|_| ()),
        ),
        ("binding_payload", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_binding_payload(instance_id, item_index)
                .map(|_| ()),
        ),
        ("slot_semantic", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_slot_semantic(instance_id, item_index)
                .map(|_| ()),
        ),
        ("layout_semantic", "selected") => Some(
            editor_manager
                .select_ui_asset_editor_layout_semantic(instance_id, item_index)
                .map(|_| ()),
        ),
        _ => None,
    }
}
