use super::super::super::super::componentized_workbench_regions::{
    authored_panes, is_authored_tree_row,
};
use super::super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::super::surface_hit_test::HostWorkbenchHitIndex;
use super::super::pane_frames::template_nodes::collect_template_node_control;
use super::UiProfileNamedFrame;

// Traverse the same indexed subtree and pane clip as native authored-pane painting.
// Fixed hierarchy preview rows are suppressed there and have their own live-row profile.
pub(super) fn collect_authored_pane_controls(
    presentation: &HostWindowPresentationData,
    out: &mut Vec<UiProfileNamedFrame>,
) {
    let index = HostWorkbenchHitIndex::from_presentation(presentation);
    for pane in authored_panes(presentation) {
        let surface = if pane.root.control_id.as_str() == "WorkbenchMainBandActivityRail" {
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
            collect_template_node_control(
                surface,
                &node,
                &FrameRect::default(),
                Some(&pane.frame),
                out,
            );
        });
    }
}

#[cfg(test)]
#[path = "authored/tests/cases.rs"]
mod tests;
