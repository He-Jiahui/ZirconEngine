use std::collections::BTreeMap;

use super::*;
use crate::ui::layouts::common::model_rc;

fn dispatchable_node() -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData::default();
    node.node_id = "status-progress".into();
    node.parent_node_id = "root".into();
    node.control_id = "WorkbenchStatusProgress".into();
    node.action_id = "workbench.status.cancel".into();
    node.frame.x = 20.0;
    node.frame.y = 30.0;
    node.frame.width = 120.0;
    node.frame.height = 24.0;
    node
}

#[test]
fn workbench_hit_index_returns_highest_z_node_independent_of_row_order() {
    let mut front = dispatchable_node();
    front.node_id = "front".into();
    front.control_id = "WorkbenchFront".into();
    front.z_index = 10;
    let mut back = dispatchable_node();
    back.node_id = "back".into();
    back.control_id = "WorkbenchBack".into();
    back.z_index = 0;

    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes = model_rc(vec![front, back]);
    let nodes = presentation.workbench_window_nodes.clone();
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let hit =
        super::super::hit::hit_test_workbench_template_nodes_with_index(&nodes, &index, 21.0, 31.0)
            .expect("overlapping workbench nodes should hit the topmost row");
    assert_eq!(hit.control_id.as_str(), "WorkbenchFront");
}

#[test]
fn geometry_patch_moves_hit_cells_without_mutating_the_old_index() {
    let mut root = TemplatePaneNodeData::default();
    root.node_id = "root".into();
    root.control_id = "UiHostWindowRoot".into();
    root.frame.width = 512.0;
    root.frame.height = 256.0;
    let button = dispatchable_node();
    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes = model_rc(vec![root, button.clone()]);
    let old_nodes = presentation.workbench_window_nodes.clone();
    let old_index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let mut moved_button = button;
    moved_button.frame.x = 180.0;
    let next_nodes = old_nodes.with_row_patches(BTreeMap::from([(1, moved_button)]));
    let mut next_presentation = presentation.clone();
    next_presentation.workbench_window_nodes = next_nodes.clone();
    let next_index = old_index
        .patch_geometry_presentation(&presentation, &next_presentation, &[1])
        .expect("stable geometry should patch the hit index");

    assert!(
        super::super::hit::hit_test_workbench_template_nodes_with_index(
            &old_nodes, &old_index, 21.0, 31.0,
        )
        .is_some()
    );
    assert!(
        super::super::hit::hit_test_workbench_template_nodes_with_index(
            &old_nodes, &old_index, 181.0, 31.0,
        )
        .is_none()
    );
    assert!(
        super::super::hit::hit_test_workbench_template_nodes_with_index(
            &next_nodes,
            &next_index,
            21.0,
            31.0,
        )
        .is_none()
    );
    let moved_hit = super::super::hit::hit_test_workbench_template_nodes_with_index(
        &next_nodes,
        &next_index,
        181.0,
        31.0,
    )
    .expect("new generation should hit the moved button");
    assert_eq!(moved_hit.control_id.as_str(), "WorkbenchStatusProgress");
}
