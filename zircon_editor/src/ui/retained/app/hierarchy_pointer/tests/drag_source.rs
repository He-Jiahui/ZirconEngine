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
        Some(HierarchyPointerRoute::Node { item_index: 1 }),
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
        Some(HierarchyPointerRoute::Node { item_index: 2 }),
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
        Some(HierarchyPointerRoute::Node { item_index: 0 }),
        &visible_entries,
        &authoritative_entries,
    )
    .unwrap();

    assert_eq!(source.node_ids, vec![3, 7]);
}

#[test]
fn hierarchy_drag_payload_uses_the_current_authoritative_row() {
    let authoritative_entries = entries();
    let mut stale_visible_entry = authoritative_entries[1].clone();
    stale_visible_entry.display_name = "Old Cube".to_string();

    let source = hierarchy_drag_source_from_route(
        Some(HierarchyPointerRoute::Node { item_index: 0 }),
        &[stale_visible_entry],
        &authoritative_entries,
    )
    .unwrap();

    assert_eq!(source.node_ids, vec![3, 7]);
    assert_eq!(
        source.payload.source.unwrap().display_name.as_deref(),
        Some("Cube")
    );
}

#[test]
fn hierarchy_reparent_target_keeps_node_and_root_targets_distinct() {
    let entries = entries();

    assert_eq!(
        hierarchy_reparent_target_from_route(
            Some(HierarchyPointerRoute::Node { item_index: 0 }),
            &entries,
        ),
        Some(Some(3))
    );
    assert_eq!(
        hierarchy_reparent_target_from_route(Some(HierarchyPointerRoute::ListSurface), &entries),
        Some(None)
    );
}
