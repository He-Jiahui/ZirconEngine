use super::super::super::super::componentized_workbench_regions::{
    authored_level_viewport, authored_panes, is_authored_tree_row,
};
use super::super::super::super::data::{
    FrameRect, HostWindowPresentationData, TemplatePaneNodeData,
};
use super::super::super::super::surface_hit_test::HostWorkbenchHitIndex;
use super::super::pane_frames::template_nodes::collect_template_node_control;
use super::UiProfileNamedFrame;

// Keep the subtree, document ownership and clip identical to authored-pane painting.
pub(super) fn collect_authored_pane_controls(
    presentation: &HostWindowPresentationData,
    out: &mut Vec<UiProfileNamedFrame>,
    viewport_toolbar_controls: &mut Vec<UiProfileNamedFrame>,
) {
    let index = HostWorkbenchHitIndex::from_presentation(presentation);
    let scene = &presentation.host_scene_data;
    for pane in authored_panes(presentation) {
        let is_viewport = pane.root.control_id.as_str() == "WorkbenchMainBandViewportPanel";
        let document = is_viewport
            .then(|| {
                scene.document_surfaces().iter().find(|document| {
                    authored_level_viewport(presentation, document)
                        .is_some_and(|viewport| viewport.root.node_id == pane.root.node_id)
                })
            })
            .flatten();
        if is_viewport && document.is_none() {
            continue;
        }
        let surface = if let Some(document) = document {
            if scene.document_leaves.is_empty() {
                "document"
            } else {
                document.surface_key.as_str()
            }
        } else if pane.root.control_id.as_str() == "WorkbenchMainBandActivityRail" {
            "left"
        } else {
            "right"
        };
        index.visit_paint_rows_for_subtree(pane.root_row, &pane.frame, &mut |row| {
            let Some(node) = presentation.workbench_window_nodes.row_data(row) else {
                return;
            };
            if is_authored_tree_row(&node) {
                return;
            }
            if is_viewport && belongs_to_viewport_toolbar(presentation, &node) {
                let previous_len = viewport_toolbar_controls.len();
                collect_template_node_control(
                    surface,
                    &node,
                    &FrameRect::default(),
                    Some(&pane.frame),
                    viewport_toolbar_controls,
                );
                if let Some(control) = viewport_toolbar_controls.get_mut(previous_len) {
                    control.id = format!("viewport_toolbar_control.{surface}.{}", node.control_id);
                    control.kind = "viewport_toolbar_control".into();
                }
            } else {
                collect_template_node_control(
                    surface,
                    &node,
                    &FrameRect::default(),
                    Some(&pane.frame),
                    out,
                );
            }
        });
    }
}

fn belongs_to_viewport_toolbar(
    presentation: &HostWindowPresentationData,
    node: &TemplatePaneNodeData,
) -> bool {
    let nodes = &presentation.workbench_window_nodes;
    let mut current = node;
    for _ in 0..nodes.row_count() {
        if current.control_id.as_str() == "UnrealViewportToolbar" {
            return true;
        }
        if current.parent_node_id.is_empty() {
            return false;
        }
        let Some(parent) = nodes
            .iter()
            .find(|candidate| candidate.node_id.as_str() == current.parent_node_id.as_str())
        else {
            return false;
        };
        current = parent;
    }
    false
}

#[cfg(test)]
#[path = "authored/tests/cases.rs"]
mod tests;
