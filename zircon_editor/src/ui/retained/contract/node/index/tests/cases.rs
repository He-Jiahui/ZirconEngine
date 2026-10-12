use std::sync::Arc;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

use super::{same_index_membership, HostWorkbenchHitIndex};
use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::TemplatePaneNodeData;

fn dispatchable_node() -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData::default();
    node.node_id = "status-progress".into();
    node.parent_node_id = "status-bar".into();
    node.control_id = "WorkbenchStatusProgress".into();
    node.action_id = "workbench.status.cancel".into();
    node.frame.x = 20.0;
    node.frame.y = 30.0;
    node.frame.width = 120.0;
    node.frame.height = 24.0;
    node
}

#[test]
fn semantic_only_row_changes_reuse_hit_membership() {
    let previous = dispatchable_node();
    let mut next = previous.clone();
    next.text = "Indexing assets".into();
    next.value_percent = 42.0;

    assert!(same_index_membership(&previous, &next));
}

#[test]
fn geometry_order_or_dispatch_changes_rebuild_hit_membership() {
    let previous = dispatchable_node();

    let mut geometry = previous.clone();
    geometry.frame.x += 1.0;
    assert!(!same_index_membership(&previous, &geometry));

    let mut order = previous.clone();
    order.z_index += 1;
    assert!(!same_index_membership(&previous, &order));

    let mut dispatch = previous.clone();
    dispatch.action_id = "".into();
    assert!(!same_index_membership(&previous, &dispatch));

    let mut tooltip = previous.clone();
    tooltip.action_id.clear();
    tooltip.disabled = true;
    tooltip.surface_node_id = Some(UiNodeId::new(77));
    tooltip.has_workbench_icon_tooltip = true;
    let mut no_tooltip = tooltip.clone();
    no_tooltip.has_workbench_icon_tooltip = false;
    assert!(!same_index_membership(&tooltip, &no_tooltip));
}

#[test]
fn tooltip_only_node_is_move_hit_but_not_press_dispatch_target() {
    let mut tooltip = dispatchable_node();
    tooltip.action_id.clear();
    tooltip.disabled = true;
    tooltip.surface_node_id = Some(UiNodeId::new(77));
    tooltip.has_workbench_icon_tooltip = true;

    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes = model_rc(vec![tooltip]);
    let nodes = presentation.workbench_window_nodes.clone();
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let move_hit = super::super::hit::hit_test_workbench_template_node_for_pointer_move_with_index(
        &nodes, &index, 21.0, 31.0,
    )
    .expect("tooltip-only node should remain a pointer-move candidate");
    assert_eq!(move_hit.surface_node_id, Some(UiNodeId::new(77)));
    assert!(!move_hit.dispatchable);
    assert!(
        super::super::hit::hit_test_workbench_template_nodes_with_index(
            &nodes, &index, 21.0, 31.0,
        )
        .is_none()
    );
}

#[test]
fn dock_paint_model_rebind_reuses_root_hit_cells_and_reindexes_only_replacements() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes = model_rc(vec![dispatchable_node()]);
    presentation.host_scene_data.left_dock.rail_nodes = model_rc(vec![paint_node("rail-old", 0.0)]);
    presentation
        .host_scene_data
        .left_dock
        .pane
        .template_v2
        .nodes = model_rc(vec![paint_node("pane-old", 64.0)]);
    let index = HostWorkbenchHitIndex::from_presentation(&presentation);

    let previous_rail = presentation.host_scene_data.left_dock.rail_nodes.clone();
    let previous_pane = presentation
        .host_scene_data
        .left_dock
        .pane
        .template_v2
        .nodes
        .clone();
    let next_rail = model_rc(vec![paint_node("rail-next", 0.0)]);
    let next_pane = model_rc(vec![paint_node("pane-next", 128.0)]);
    presentation.host_scene_data.left_dock.rail_nodes = next_rail.clone();
    presentation
        .host_scene_data
        .left_dock
        .pane
        .template_v2
        .nodes = next_pane.clone();

    let rebound = index
        .rebind_paint_models(&[(previous_rail, next_rail), (previous_pane, next_pane)])
        .expect("stable dock model cardinality should support a local rebind");

    assert!(Arc::ptr_eq(&index.buckets, &rebound.buckets));
    assert!(rebound.indexes_presentation(&presentation));
}

#[test]
fn paint_index_streams_build_time_order_and_reuses_multi_cell_scratch() {
    let nodes = model_rc(vec![
        ordered_paint_node("back", 20),
        ordered_paint_node("front", 5),
        ordered_paint_node("middle", 10),
    ]);
    let index = super::HostTemplateNodePaintIndex::new(nodes);

    assert_eq!(
        index.rows_for_clip(&super::FrameRect {
            x: 1.0,
            y: 1.0,
            width: 8.0,
            height: 8.0,
        }),
        vec![1, 2, 0]
    );
    assert_eq!(index.query_sort_count_for_test(), 0);

    assert_eq!(
        index.rows_for_clip(&super::FrameRect {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 24.0,
        }),
        vec![1, 2, 0]
    );
    assert_eq!(index.query_sort_count_for_test(), 0);

    let multi_cell_index = super::HostTemplateNodePaintIndex::new(model_rc(vec![
        paint_node("left", 0.0),
        paint_node("middle", 70.0),
        paint_node("right", 140.0),
    ]));
    let scratch_capacity = multi_cell_index.query_scratch_capacity_for_test();
    let multi_cell_clip = super::FrameRect {
        x: 1.0,
        y: 0.0,
        width: 170.0,
        height: 24.0,
    };

    assert_eq!(
        multi_cell_index.rows_for_clip(&multi_cell_clip),
        vec![0, 1, 2]
    );
    assert_eq!(
        multi_cell_index.rows_for_clip(&multi_cell_clip),
        vec![0, 1, 2]
    );
    assert_eq!(
        multi_cell_index.query_scratch_capacity_for_test(),
        scratch_capacity
    );
    assert_eq!(
        multi_cell_index.query_scratch_allocation_count_for_test(),
        0
    );
}

fn paint_node(id: &str, x: f32) -> TemplatePaneNodeData {
    let mut node = TemplatePaneNodeData::default();
    node.node_id = id.into();
    node.control_id = id.into();
    node.frame.x = x;
    node.frame.width = 32.0;
    node.frame.height = 24.0;
    node
}

fn ordered_paint_node(id: &str, z_index: i32) -> TemplatePaneNodeData {
    let mut node = paint_node(id, 0.0);
    node.z_index = z_index;
    node
}
