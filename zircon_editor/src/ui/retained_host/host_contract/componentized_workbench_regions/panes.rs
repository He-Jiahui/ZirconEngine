use super::super::data::{FrameRect, HostWindowPresentationData, PaneData, TemplatePaneNodeData};
use crate::ui::retained_host::hierarchy_pointer::HierarchyRowMetrics;

pub(in crate::ui::retained_host) fn owns_ordinary_panes(
    presentation: &HostWindowPresentationData,
) -> bool {
    presentation
        .workbench_window_nodes
        .iter()
        .any(|node| node.control_id.as_str() == "WorkbenchSceneWorkspace")
}

pub(in crate::ui::retained_host) const PANE_ROOT_CONTROLS: &[&str] = &[
    "WorkbenchMainBandActivityRail",
    "WorkbenchMainBandSceneTreePanel",
    "WorkbenchMainBandInspectorPanel",
];

pub(in crate::ui::retained_host) struct AuthoredPane<'a> {
    pub root_row: usize,
    pub root: &'a TemplatePaneNodeData,
    pub frame: FrameRect,
}

pub(in crate::ui::retained_host) fn authored_panes(
    presentation: &HostWindowPresentationData,
) -> Vec<AuthoredPane<'_>> {
    // These are the same absolute projected node frames and inherited clips used
    // by the shared template painter and hit index. The actual root bounds avoid
    // treating an off-window preferred-size pane as a visible owner.
    let nodes = &presentation.workbench_window_nodes;
    let Some(bounds) = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "WorkbenchWindowRoot")
        .map(node_frame)
    else {
        return Vec::new();
    };
    nodes
        .iter()
        .enumerate()
        .filter_map(|(root_row, root)| {
            if !PANE_ROOT_CONTROLS.contains(&root.control_id.as_str()) {
                return None;
            }
            let mut frame = super::intersect_rect(&node_frame(root), &bounds)?;
            if root.has_clip_frame {
                let inherited_clip = FrameRect {
                    x: root.clip_frame.x,
                    y: root.clip_frame.y,
                    width: root.clip_frame.width,
                    height: root.clip_frame.height,
                };
                frame = super::intersect_rect(&frame, &inherited_clip)?;
            }
            Some(AuthoredPane {
                root_row,
                root,
                frame,
            })
        })
        .collect()
}

pub(in crate::ui::retained_host) struct AuthoredHierarchy<'a> {
    pub pane: &'a PaneData,
    pub viewport: FrameRect,
    pub prototype: &'a TemplatePaneNodeData,
    pub metrics: HierarchyRowMetrics,
}

/// The complete runtime hierarchy stays the data owner. The authored viewport and row
/// prototype provide geometry and style to its bounded native row renderer and input bridge.
pub(in crate::ui::retained_host) fn authored_hierarchy(
    presentation: &HostWindowPresentationData,
) -> Option<AuthoredHierarchy<'_>> {
    let nodes = &presentation.workbench_window_nodes;
    let viewport = nodes
        .iter()
        .find(|n| {
            n.control_id.as_str() == "WorkbenchSceneTree"
                && n.frame.width > 0.0
                && n.frame.height > 0.0
        })
        .map(node_frame)?;
    let prototype = nodes
        .iter()
        .find(|n| is_authored_tree_row(n) && n.frame.height > 0.0)?;
    let next = nodes
        .iter()
        .filter(|n| is_authored_tree_row(n) && n.frame.y > prototype.frame.y)
        .map(|n| n.frame.y)
        .min_by(f32::total_cmp);
    let gap = next
        .map(|y| (y - prototype.frame.y - prototype.frame.height).max(0.0))
        .unwrap_or_else(|| {
            crate::ui::retained_host::host_contract::paint_theme::current_host_metrics()
                .border_width
        });
    let pane = [
        &presentation.host_scene_data.left_dock.pane,
        &presentation.host_scene_data.right_dock.pane,
    ]
    .into_iter()
    .find(|pane| pane.kind.as_str() == "Hierarchy")?;
    Some(AuthoredHierarchy {
        pane,
        metrics: HierarchyRowMetrics {
            row_x: (prototype.frame.x - viewport.x).max(0.0),
            row_y: (prototype.frame.y - viewport.y).max(0.0),
            row_height: prototype.frame.height,
            row_gap: gap,
            row_width_inset: (viewport.width - prototype.frame.width).max(0.0),
        },
        viewport,
        prototype,
    })
}

pub(in crate::ui::retained_host) fn is_authored_tree_row(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        "WorkbenchSceneRootItem"
            | "WorkbenchSceneEnvironmentItem"
            | "WorkbenchSceneLevelItem"
            | "WorkbenchScenePropsItem"
            | "WorkbenchScenePlayerItem"
            | "WorkbenchSceneAudioItem"
            | "WorkbenchSceneSlot07Item"
            | "WorkbenchSceneSlot08Item"
            | "WorkbenchSceneSlot09Item"
            | "WorkbenchSceneSlot10Item"
    )
}

pub(in crate::ui::retained_host) fn point_in_frame(frame: &FrameRect, x: f32, y: f32) -> bool {
    super::super::frame_geometry::contains_point(frame, x, y)
}

fn node_frame(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

#[cfg(test)]
#[path = "tests/panes.rs"]
mod tests;
