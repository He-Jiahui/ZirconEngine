use super::*;
use crate::ui::retained_host::host_contract::componentized_workbench_regions::{
    module_tools_menu_spec, module_tools_popups, ModuleToolsPopup,
};

const WINDOW_ROOT: &str = "WorkbenchWindowRoot";
const WINDOW_CONTENT: &str = "UnrealLevelEditorContent";

/// Direct Overlay siblings retain their authored frames and clips. Ordinary content
/// belongs to its chrome/pane passes; opened root siblings belong to one foreground pass.
pub(super) struct RootOverlayRows {
    rows: HashSet<usize>,
    module_popups: Vec<ModuleToolsPopup>,
    exclude: bool,
}

impl RootOverlayRows {
    pub(super) fn chrome_exclusion(presentation: &HostWindowPresentationData) -> Self {
        Self::from_presentation(presentation, false, true)
    }

    fn from_presentation(
        presentation: &HostWindowPresentationData,
        opened_only: bool,
        exclude: bool,
    ) -> Self {
        let nodes = &presentation.workbench_window_nodes;
        let Some(root) = nodes
            .iter()
            .find(|node| node.control_id.as_str() == WINDOW_ROOT)
        else {
            return Self {
                rows: HashSet::new(),
                module_popups: Vec::new(),
                exclude,
            };
        };
        let nodes_by_id = node_index(nodes);
        let overlay_roots: Vec<&str> = nodes
            .iter()
            .filter(|node| {
                node.parent_node_id == root.node_id
                    && node.control_id.as_str() != WINDOW_CONTENT
                    && (!opened_only
                        || node.popup_open
                        || node.control_id.as_str() == "UnrealContentDrawerRegion"
                        || nodes.iter().any(|child| {
                            child.popup_open
                                && reaches_subtree_root(
                                    child.node_id.as_str(),
                                    node.node_id.as_str(),
                                    &nodes_by_id,
                                )
                        }))
            })
            .map(|node| node.node_id.as_str())
            .collect();
        let mut rows: HashSet<usize> = nodes
            .iter()
            .enumerate()
            .filter_map(|(row, node)| {
                overlay_roots
                    .iter()
                    .any(|root| reaches_subtree_root(node.node_id.as_str(), root, &nodes_by_id))
                    .then_some(row)
            })
            .collect();
        let bounds = FrameRect {
            x: root.frame.x,
            y: root.frame.y,
            width: root.frame.width,
            height: root.frame.height,
        };
        let module_popups = module_tools_popups(presentation, &bounds);
        if exclude {
            // Known menu sources never fall back to the chrome/pane painter,
            // including closed or unproven incarnations.
            rows.extend(nodes.iter().enumerate().filter_map(|(row, node)| {
                module_tools_menu_spec(node.control_id.as_str()).map(|_| row)
            }));
        } else {
            rows.extend(module_popups.iter().map(|popup| popup.row));
        }
        Self {
            rows,
            module_popups,
            exclude,
        }
    }
}

impl TemplateNodePaintTransform for RootOverlayRows {
    fn stream_row_visit_indices(
        &self,
        _row_count: usize,
        _clip: &FrameRect,
        visit: &mut dyn FnMut(usize),
    ) -> bool {
        if self.exclude {
            return false;
        }
        let mut rows: Vec<usize> = self.rows.iter().copied().collect();
        rows.sort_unstable();
        for row in rows {
            visit(row);
        }
        true
    }

    fn transform_row(
        &self,
        row: usize,
        mut node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        if self.rows.contains(&row) == self.exclude {
            return None;
        }
        if !self.exclude {
            if let Some(popup) = self.module_popups.iter().find(|popup| popup.row == row) {
                let clip = intersect_rect(&clip, &popup.clip)?;
                node.has_clip_frame = true;
                node.clip_frame =
                    crate::ui::retained_host::host_contract::data::TemplateNodeFrameData {
                        x: clip.x,
                        y: clip.y,
                        width: clip.width,
                        height: clip.height,
                    };
                return Some((node, clip));
            }
        }
        Some((node, clip))
    }
}

pub(super) fn draw_authored_root_overlays(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    bounds: &FrameRect,
) {
    // With no authored regions, the existing whole-window fallback already paints overlays.
    if componentized_workbench_chrome_regions(&presentation.host_layout, bounds).is_none() {
        return;
    }
    let transform = RootOverlayRows::from_presentation(presentation, true, false);
    if transform.rows.is_empty() {
        return;
    }
    let focus = paint_text_input_focus(presentation);
    draw_template_nodes_with_transform(
        frame,
        &presentation.workbench_window_nodes,
        &zero_origin(),
        bounds,
        Some(&focus),
        Some(&transform),
    );
}

#[cfg(test)]
#[path = "../../workbench/overlay/tests/cases.rs"]
mod tests;
