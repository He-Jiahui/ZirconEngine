use super::super::{SceneEntry, UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata};
use crate::ui::retained_host::hierarchy_pointer::HierarchyPointerRoute;

pub(super) fn scene_drag_payload_from_route(
    route: Option<HierarchyPointerRoute>,
    scene_entries: &[SceneEntry],
) -> Option<UiDragPayload> {
    let HierarchyPointerRoute::Node { node_id, .. } = route? else {
        return None;
    };
    scene_entries
        .iter()
        .find(|entry| entry.id.to_string() == node_id)
        .map(scene_drag_payload_from_entry)
}

fn scene_drag_payload_from_entry(entry: &SceneEntry) -> UiDragPayload {
    let reference = format!("scene://node/{}", entry.id);
    UiDragPayload::new(UiDragPayloadKind::SceneInstance, reference.clone()).with_source(
        UiDragSourceMetadata {
            source_surface: "hierarchy".to_string(),
            source_control_id: "HierarchyListPanel".to_string(),
            locator: Some(reference),
            display_name: Some(entry.name.clone()),
            asset_kind: Some("Scene Instance".to_string()),
            ..UiDragSourceMetadata::default()
        },
    )
}
