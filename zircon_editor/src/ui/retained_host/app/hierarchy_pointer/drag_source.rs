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
    let node_ids = if authoritative_scene_entries.is_selected(entry.entity) {
        authoritative_scene_entries
            .iter()
            .filter(|entry| authoritative_scene_entries.is_selected(entry.entity))
            .map(|entry| entry.entity)
            .collect()
    } else {
        vec![entry.entity]
    };
    Some(HierarchyDragSource {
        node_ids,
        payload: scene_drag_payload_from_entry(entry),
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
mod tests {
    use super::{hierarchy_drag_source_from_route, hierarchy_reparent_target_from_route};
    use crate::ui::retained_host::hierarchy_pointer::HierarchyPointerRoute;
    use crate::ui::workbench::snapshot::{SceneEntries, SceneEntry};

    fn entries() -> SceneEntries {
        SceneEntries::from_entries(
            vec![
                SceneEntry {
                    id: 3,
                    name: "Camera".to_string(),
                    depth: 0,
                },
                SceneEntry {
                    id: 7,
                    name: "Cube".to_string(),
                    depth: 0,
                },
                SceneEntry {
                    id: 11,
                    name: "Light".to_string(),
                    depth: 0,
                },
            ],
            [3, 7],
        )
    }

    #[test]
    fn selected_hierarchy_drag_preserves_the_full_selection() {
        let entries = entries();
        let source = hierarchy_drag_source_from_route(
            Some(HierarchyPointerRoute::Node {
                item_index: 1,
                node_id: "7".to_string(),
            }),
            &entries,
            &entries,
        )
        .unwrap();

        assert_eq!(source.node_ids, vec![3, 7]);
        assert_eq!(source.payload.reference, "scene://node/7");
    }

    #[test]
    fn unselected_hierarchy_drag_uses_only_the_pressed_node() {
        let entries = entries();
        let source = hierarchy_drag_source_from_route(
            Some(HierarchyPointerRoute::Node {
                item_index: 2,
                node_id: "11".to_string(),
            }),
            &entries,
            &entries,
        )
        .unwrap();

        assert_eq!(source.node_ids, vec![11]);
    }

    #[test]
    fn selected_hierarchy_drag_uses_the_authoritative_selection_beyond_the_filtered_projection() {
        let authoritative_entries = entries();
        let visible_entries = vec![authoritative_entries[1].clone()];

        let source = hierarchy_drag_source_from_route(
            Some(HierarchyPointerRoute::Node {
                item_index: 0,
                node_id: "7".to_string(),
            }),
            &visible_entries,
            &authoritative_entries,
        )
        .unwrap();

        assert_eq!(source.node_ids, vec![3, 7]);
    }

    #[test]
    fn hierarchy_reparent_target_keeps_node_and_root_targets_distinct() {
        let entries = entries();

        assert_eq!(
            hierarchy_reparent_target_from_route(
                Some(HierarchyPointerRoute::Node {
                    item_index: 0,
                    node_id: "3".to_string(),
                }),
                &entries,
            ),
            Some(Some(3))
        );
        assert_eq!(
            hierarchy_reparent_target_from_route(
                Some(HierarchyPointerRoute::ListSurface),
                &entries
            ),
            Some(None)
        );
    }
}
