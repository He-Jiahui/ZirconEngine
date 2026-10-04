use super::*;

const WINDOW_ROOT: &str = "WorkbenchWindowRoot";
const WINDOW_CONTENT: &str = "WorkbenchWindowContent";

/// Direct Overlay siblings retain their authored frames and clips. Ordinary content
/// belongs to its chrome/pane passes; opened root siblings belong to one foreground pass.
pub(super) struct RootOverlayRows {
    rows: HashSet<usize>,
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
                exclude,
            };
        };
        let overlay_roots: Vec<&str> = nodes
            .iter()
            .filter(|node| {
                node.parent_node_id == root.node_id
                    && node.control_id.as_str() != WINDOW_CONTENT
                    && (!opened_only || node.popup_open)
            })
            .map(|node| node.node_id.as_str())
            .collect();
        let nodes_by_id = node_index(nodes);
        let rows = nodes
            .iter()
            .enumerate()
            .filter_map(|(row, node)| {
                overlay_roots
                    .iter()
                    .any(|root| reaches_subtree_root(node.node_id.as_str(), root, &nodes_by_id))
                    .then_some(row)
            })
            .collect();
        Self { rows, exclude }
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
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        (self.rows.contains(&row) != self.exclude).then_some((node, clip))
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
#[path = "root_overlays/tests/cases.rs"]
mod tests;
