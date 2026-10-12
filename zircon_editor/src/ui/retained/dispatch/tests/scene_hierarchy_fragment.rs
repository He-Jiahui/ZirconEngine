use std::collections::BTreeMap;

use super::{rows_visible_with_expansion, BuiltinWorkbenchWindowTemplateSurfaceBridge};
use zircon_runtime::scene::WorldInspectionHierarchyRow;
use zircon_runtime_interface::ui::component::UiValue;
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::workbench::snapshot::SceneEntries;

fn row(
    entity: u64,
    parent: Option<u64>,
    depth: u32,
    has_children: bool,
) -> WorldInspectionHierarchyRow {
    WorldInspectionHierarchyRow {
        entity,
        parent,
        depth,
        display_name: entity.to_string(),
        kind: "Entity".to_string(),
        subtree_hash: 0,
        active_in_hierarchy: true,
        has_children,
    }
}

#[test]
fn disclosure_state_is_keyed_by_entity_when_row_slots_are_reassigned() {
    let rows = [
        row(30, None, 0, true),
        row(31, Some(30), 1, false),
        row(10, None, 0, true),
        row(11, Some(10), 1, false),
    ];
    let expanded = BTreeMap::from([(10, true), (30, false)]);

    let visible = rows_visible_with_expansion(&rows, &expanded);

    assert_eq!(
        visible.iter().map(|row| row.entity).collect::<Vec<_>>(),
        [30, 10, 11]
    );
}

#[test]
fn scene_surface_preserves_entity_expansion_across_filtering_and_slot_reuse() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    let initial = SceneEntries::from_hierarchy_rows_at_generation(
        vec![row(10, None, 0, true), row(11, Some(10), 1, false)],
        [],
        1,
    );
    bridge.sync_scene_view_state(&initial, "", &[]).unwrap();
    assert!(!bridge.scene_row_expanded("WorkbenchSceneRootItem"));
    bridge
        .mutate_control_property_for_test("WorkbenchSceneRootItem", "expanded", UiValue::Bool(true))
        .unwrap();
    assert!(bridge.scene_row_expanded("WorkbenchSceneRootItem"));

    let reordered = SceneEntries::from_hierarchy_rows_at_generation(
        vec![
            row(30, None, 0, true),
            row(31, Some(30), 1, false),
            row(10, None, 0, true),
            row(11, Some(10), 1, false),
        ],
        [],
        2,
    );
    bridge.set_scene_filter_query("");
    bridge.resync_scene_hierarchy(&reordered).unwrap();

    assert_eq!(
        bridge.scene_node_id_for_control("WorkbenchSceneRootItem"),
        Some(30)
    );
    assert!(!bridge.scene_row_expanded("WorkbenchSceneRootItem"));
    assert_eq!(
        bridge.scene_node_id_for_control("WorkbenchSceneEnvironmentItem"),
        Some(10)
    );
    assert!(bridge.scene_row_expanded("WorkbenchSceneEnvironmentItem"));
    assert_eq!(bridge.effective_expanded_ids(&reordered), [10]);

    bridge.set_scene_filter_query("31");
    bridge.resync_scene_hierarchy(&reordered).unwrap();
    assert!(bridge.scene_row_expanded("WorkbenchSceneRootItem"));
    assert_eq!(bridge.effective_expanded_ids(&reordered), [10]);

    bridge.set_scene_filter_query("");
    bridge.resync_scene_hierarchy(&reordered).unwrap();
    assert!(!bridge.scene_row_expanded("WorkbenchSceneRootItem"));
    assert!(bridge.scene_row_expanded("WorkbenchSceneEnvironmentItem"));
    assert_eq!(bridge.effective_expanded_ids(&reordered), [10]);
}
