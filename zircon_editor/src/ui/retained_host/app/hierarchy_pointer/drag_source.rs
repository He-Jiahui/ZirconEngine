use zircon_runtime::scene::{NodeId, WorldInspectionHierarchyRow};

use super::super::{SceneEntries, UiDragPayload, UiDragPayloadKind, UiDragSourceMetadata};
use crate::ui::retained_host::hierarchy_pointer::HierarchyPointerRoute;

pub(super) struct HierarchyDragSource {
    pub(super) node_ids: Vec<NodeId>,
    pub(super) payload: UiDragPayload,
}

pub(super) fn hierarchy_drag_source_from_route(
    route: Option<HierarchyPointerRoute>,
    scene_entries: &[WorldInspectionHierarchyRow],
    authoritative_scene_entries: &SceneEntries,
) -> Option<HierarchyDragSource> {
    let HierarchyPointerRoute::Node { item_index, .. } = route? else {
        return None;
    };
    let entry = scene_entries.get(item_index)?;
    let authoritative_entry = authoritative_scene_entries
        .iter()
        .find(|candidate| candidate.entity == entry.entity)?;
    let node_ids = if authoritative_scene_entries.is_selected(authoritative_entry.entity) {
        authoritative_scene_entries
            .iter()
            .filter(|entry| authoritative_scene_entries.is_selected(entry.entity))
            .map(|entry| entry.entity)
            .collect()
    } else {
        vec![authoritative_entry.entity]
    };
    Some(HierarchyDragSource {
        node_ids,
        payload: scene_drag_payload_from_entry(authoritative_entry),
    })
}

pub(super) fn hierarchy_reparent_target_from_route(
    route: Option<HierarchyPointerRoute>,
    scene_entries: &[WorldInspectionHierarchyRow],
) -> Option<Option<NodeId>> {
    match route? {
        HierarchyPointerRoute::Node { item_index, .. } => scene_entries
            .get(item_index)
            .map(|entry| Some(entry.entity)),
        HierarchyPointerRoute::ListSurface => Some(None),
    }
}

fn scene_drag_payload_from_entry(entry: &WorldInspectionHierarchyRow) -> UiDragPayload {
    let reference = format!("scene://node/{}", entry.entity);
    UiDragPayload::new(UiDragPayloadKind::SceneInstance, reference.clone()).with_source(
        UiDragSourceMetadata {
            source_surface: "hierarchy".to_string(),
            source_control_id: "HierarchyListPanel".to_string(),
            locator: Some(reference),
            display_name: Some(entry.display_name.clone()),
            asset_kind: Some("Scene Instance".to_string()),
            ..UiDragSourceMetadata::default()
        },
    )
}

#[cfg(test)]
#[path = "tests/drag_source.rs"]
mod tests;
