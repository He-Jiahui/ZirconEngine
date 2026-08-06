use std::collections::{HashMap, HashSet};

#[cfg(test)]
use std::cell::Cell;

use super::super::super::data::{
    FrameRect, HostWindowPresentationData, PaneData, TemplatePaneNodeData,
};
use super::super::super::template_geometry::template_nodes_bounds;
use super::surface_frame_builder::is_dispatchable;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

const HIT_INDEX_CELL_SIZE: f32 = 64.0;
const EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID: &str = "WorkbenchExtensionModuleWorkspacesHost";

#[derive(Clone, Debug)]
pub(crate) struct HostExtensionWorkspacePaintIndex {
    pub(crate) root_row: usize,
    pub(crate) root_node_id: String,
    pub(crate) host_frame: FrameRect,
}

pub(crate) struct HostWorkbenchHitIndex {
    indexed_nodes: ModelRc<TemplatePaneNodeData>,
    origin: Option<FrameRect>,
    buckets: HashMap<(i32, i32), Vec<usize>>,
    paint_indices: Vec<HostTemplateNodePaintIndex>,
    popup_rows: Vec<usize>,
    parent_rows: Vec<Option<usize>>,
    extension_workspace: Option<HostExtensionWorkspacePaintIndex>,
    #[cfg(test)]
    last_candidate_visit_count: Cell<usize>,
}

struct HostTemplateNodePaintIndex {
    indexed_nodes: ModelRc<TemplatePaneNodeData>,
    origin: Option<FrameRect>,
    buckets: HashMap<(i32, i32), Vec<usize>>,
}

impl HostWorkbenchHitIndex {
    pub(crate) fn from_presentation(presentation: &HostWindowPresentationData) -> Self {
        record_current_ui_perf_counter(UiPerfCounter::WorkbenchHitIndexBuildCount, 1.0);
        let nodes = &presentation.workbench_window_nodes;
        let rows_by_node_id = nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| !node.node_id.is_empty())
            .map(|(row, node)| (node.node_id.as_str(), row))
            .collect::<HashMap<_, _>>();
        let parent_rows = nodes
            .iter()
            .map(|node| rows_by_node_id.get(node.parent_node_id.as_str()).copied())
            .collect::<Vec<_>>();
        let extension_workspace = extension_workspace_index(nodes, &parent_rows);
        let origin = template_nodes_bounds(nodes).map(|bounds| FrameRect {
            x: 0.0,
            y: 0.0,
            width: bounds.width.max(bounds.x + bounds.width).max(1.0),
            height: bounds.height.max(bounds.y + bounds.height).max(1.0),
        });
        let mut index = Self {
            indexed_nodes: nodes.clone(),
            origin,
            buckets: HashMap::new(),
            paint_indices: presentation_paint_node_models(presentation)
                .into_iter()
                .map(HostTemplateNodePaintIndex::new)
                .collect(),
            popup_rows: Vec::new(),
            parent_rows,
            extension_workspace,
            #[cfg(test)]
            last_candidate_visit_count: Cell::new(0),
        };
        let Some(origin) = index.origin.clone() else {
            return index;
        };
        for (row, node) in nodes.iter().enumerate() {
            if node.popup_open && !node.disabled && !node.control_id.is_empty() {
                index.popup_rows.push(row);
            }
            let Some(frame) = indexed_node_frame(node, &origin) else {
                continue;
            };
            if is_dispatchable(node) {
                insert_frame(&mut index.buckets, row, &frame);
            }
        }
        index
    }

    pub(super) fn origin(&self) -> Option<&FrameRect> {
        self.origin.as_ref()
    }

    pub(super) fn popup_rows(&self) -> &[usize] {
        &self.popup_rows
    }

    pub(crate) fn has_popup_rows(&self) -> bool {
        !self.popup_rows.is_empty()
    }

    pub(crate) fn indexes_nodes(&self, nodes: &ModelRc<TemplatePaneNodeData>) -> bool {
        self.indexed_nodes.shares_values_with(nodes)
    }

    pub(crate) fn indexes_paint_nodes(&self, nodes: &ModelRc<TemplatePaneNodeData>) -> bool {
        self.paint_indices
            .iter()
            .any(|index| index.indexed_nodes.shares_values_with(nodes))
    }

    pub(crate) fn indexes_presentation(&self, presentation: &HostWindowPresentationData) -> bool {
        let models = presentation_paint_node_models(presentation);
        models.len() == self.paint_indices.len()
            && models
                .iter()
                .zip(&self.paint_indices)
                .all(|(nodes, index)| index.indexed_nodes.shares_values_with(nodes))
    }

    pub(crate) fn extension_workspace(&self) -> Option<&HostExtensionWorkspacePaintIndex> {
        self.extension_workspace.as_ref()
    }

    pub(crate) fn paint_rows_for_clip(&self, clip: &FrameRect) -> Vec<usize> {
        record_current_ui_perf_counter(UiPerfCounter::WorkbenchPaintIndexQueryCount, 1.0);
        let rows = self
            .paint_indices
            .first()
            .map(|index| index.rows_for_clip(clip))
            .unwrap_or_default();
        record_current_ui_perf_counter(
            UiPerfCounter::WorkbenchPaintIndexCandidateCount,
            rows.len() as f64,
        );
        rows
    }

    pub(crate) fn paint_rows_for_nodes(
        &self,
        nodes: &ModelRc<TemplatePaneNodeData>,
        clip: &FrameRect,
    ) -> Option<Vec<usize>> {
        let index = self
            .paint_indices
            .iter()
            .find(|index| index.indexed_nodes.shares_values_with(nodes))?;
        record_current_ui_perf_counter(UiPerfCounter::WorkbenchPaintIndexQueryCount, 1.0);
        let rows = index.rows_for_clip(clip);
        record_current_ui_perf_counter(
            UiPerfCounter::WorkbenchPaintIndexCandidateCount,
            rows.len() as f64,
        );
        Some(rows)
    }

    pub(crate) fn paint_rows_for_subtree(&self, root_row: usize, clip: &FrameRect) -> Vec<usize> {
        let mut rows = self.paint_rows_for_clip(clip);
        rows.retain(|row| reaches_ancestor(*row, root_row, &self.parent_rows));
        rows
    }

    pub(super) fn candidate_rows(&self, x: f32, y: f32) -> &[usize] {
        self.buckets
            .get(&(cell_coordinate(x), cell_coordinate(y)))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(super) fn begin_query(&self) {
        #[cfg(test)]
        self.last_candidate_visit_count.set(0);
    }

    pub(super) fn record_candidate_visit(&self) {
        #[cfg(test)]
        self.last_candidate_visit_count
            .set(self.last_candidate_visit_count.get().saturating_add(1));
    }

    #[cfg(test)]
    pub(crate) fn last_candidate_visit_count_for_test(&self) -> usize {
        self.last_candidate_visit_count.get()
    }
}

impl HostTemplateNodePaintIndex {
    fn new(indexed_nodes: ModelRc<TemplatePaneNodeData>) -> Self {
        let origin = template_nodes_bounds(&indexed_nodes).map(|bounds| FrameRect {
            x: 0.0,
            y: 0.0,
            width: bounds.width.max(bounds.x + bounds.width).max(1.0),
            height: bounds.height.max(bounds.y + bounds.height).max(1.0),
        });
        let mut index = Self {
            indexed_nodes,
            origin,
            buckets: HashMap::new(),
        };
        let Some(origin) = index.origin.clone() else {
            return index;
        };
        for (row, node) in index.indexed_nodes.iter().enumerate() {
            if let Some(frame) = indexed_node_frame(node, &origin) {
                insert_frame(&mut index.buckets, row, &frame);
            }
        }
        index
    }

    fn rows_for_clip(&self, clip: &FrameRect) -> Vec<usize> {
        let Some(origin) = self.origin.as_ref() else {
            return Vec::new();
        };
        let Some(clip) = intersect_frames(clip, origin) else {
            return Vec::new();
        };
        if frame_contains(&clip, origin) {
            let mut rows = (0..self.indexed_nodes.row_count()).collect::<Vec<_>>();
            self.sort_rows_in_paint_order(&mut rows);
            return rows;
        }

        let min_x = cell_coordinate(clip.x);
        let max_x = cell_coordinate(clip.x + clip.width - f32::EPSILON);
        let min_y = cell_coordinate(clip.y);
        let max_y = cell_coordinate(clip.y + clip.height - f32::EPSILON);
        let mut seen = HashSet::new();
        let mut rows = Vec::new();
        for cell_y in min_y..=max_y {
            for cell_x in min_x..=max_x {
                if let Some(bucket) = self.buckets.get(&(cell_x, cell_y)) {
                    for row in bucket {
                        if seen.insert(*row) {
                            rows.push(*row);
                        }
                    }
                }
            }
        }
        self.sort_rows_in_paint_order(&mut rows);
        rows
    }

    fn sort_rows_in_paint_order(&self, rows: &mut [usize]) {
        rows.sort_unstable_by_key(|row| {
            (
                self.indexed_nodes
                    .get(*row)
                    .map(|node| node.z_index)
                    .unwrap_or_default(),
                *row,
            )
        });
    }
}

fn presentation_paint_node_models(
    presentation: &HostWindowPresentationData,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    let mut models = vec![presentation.workbench_window_nodes.clone()];
    push_paint_model(&mut models, &presentation.root_template_nodes);

    let scene = &presentation.host_scene_data;
    push_paint_model(&mut models, &scene.menu_chrome.template_nodes);
    push_paint_model(&mut models, &scene.page_chrome.template_nodes);
    push_paint_model(&mut models, &scene.status_bar.template_nodes);
    for menu in scene.menu_chrome.menus.iter() {
        push_paint_model(&mut models, &menu.popup_nodes);
    }

    push_paint_model(&mut models, &scene.left_dock.rail_nodes);
    push_paint_model(&mut models, &scene.left_dock.header_nodes);
    push_pane_paint_model(&mut models, &scene.left_dock.pane);
    push_paint_model(&mut models, &scene.document_dock.header_nodes);
    push_pane_paint_model(&mut models, &scene.document_dock.pane);
    push_paint_model(&mut models, &scene.right_dock.rail_nodes);
    push_paint_model(&mut models, &scene.right_dock.header_nodes);
    push_pane_paint_model(&mut models, &scene.right_dock.pane);
    push_paint_model(&mut models, &scene.bottom_dock.header_nodes);
    push_pane_paint_model(&mut models, &scene.bottom_dock.pane);

    for window in scene.floating_layer.floating_windows.iter() {
        push_paint_model(&mut models, &window.header_nodes);
        push_pane_paint_model(&mut models, &window.active_pane);
    }
    for window in presentation
        .native_floating_surface_data
        .floating_windows
        .iter()
    {
        push_paint_model(&mut models, &window.header_nodes);
        push_pane_paint_model(&mut models, &window.active_pane);
    }
    models
}

fn push_pane_paint_model(models: &mut Vec<ModelRc<TemplatePaneNodeData>>, pane: &PaneData) {
    if let Some(nodes) = super::pane_nodes::pane_template_nodes(pane) {
        push_paint_model(models, nodes);
    }
}

fn push_paint_model(
    models: &mut Vec<ModelRc<TemplatePaneNodeData>>,
    nodes: &ModelRc<TemplatePaneNodeData>,
) {
    if nodes.row_count() == 0
        || models
            .iter()
            .any(|existing| existing.shares_values_with(nodes))
    {
        return;
    }
    models.push(nodes.clone());
}

fn insert_frame(buckets: &mut HashMap<(i32, i32), Vec<usize>>, row: usize, frame: &FrameRect) {
    let min_x = cell_coordinate(frame.x);
    let max_x = cell_coordinate(frame.x + frame.width - f32::EPSILON);
    let min_y = cell_coordinate(frame.y);
    let max_y = cell_coordinate(frame.y + frame.height - f32::EPSILON);
    for cell_y in min_y..=max_y {
        for cell_x in min_x..=max_x {
            buckets.entry((cell_x, cell_y)).or_default().push(row);
        }
    }
}

fn extension_workspace_index(
    nodes: &ModelRc<TemplatePaneNodeData>,
    parent_rows: &[Option<usize>],
) -> Option<HostExtensionWorkspacePaintIndex> {
    let module_host_row = nodes
        .iter()
        .position(|node| node.control_id.as_str() == EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID)?;
    let root_row = nodes.iter().enumerate().find_map(|(row, node)| {
        let parent_row = parent_rows.get(row).copied().flatten()?;
        let parent = nodes.get(parent_row)?;
        (is_extension_workspace_root_control(node.control_id.as_str())
            && is_extension_workspace_host_control(parent.control_id.as_str())
            && reaches_ancestor(row, module_host_row, parent_rows))
        .then_some(row)
    })?;
    let module_host = nodes.get(module_host_row)?;
    let root = nodes.get(root_row)?;
    Some(HostExtensionWorkspacePaintIndex {
        root_row,
        root_node_id: root.node_id.clone(),
        host_frame: FrameRect {
            x: module_host.frame.x,
            y: module_host.frame.y,
            width: module_host.frame.width,
            height: module_host.frame.height,
        },
    })
}

fn reaches_ancestor(row: usize, ancestor_row: usize, parent_rows: &[Option<usize>]) -> bool {
    let mut current = Some(row);
    let mut remaining = parent_rows.len().saturating_add(1);
    while let Some(row) = current {
        if row == ancestor_row {
            return true;
        }
        if remaining == 0 {
            return false;
        }
        remaining -= 1;
        current = parent_rows.get(row).copied().flatten();
    }
    false
}

fn is_extension_workspace_root_control(control_id: &str) -> bool {
    control_id.starts_with("WorkbenchExtension") && control_id.ends_with("Workspace")
}

fn is_extension_workspace_host_control(control_id: &str) -> bool {
    control_id.starts_with("WorkbenchExtension") && control_id.ends_with("WorkspaceHost")
}

fn frame_contains(outer: &FrameRect, inner: &FrameRect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.width >= inner.x + inner.width
        && outer.y + outer.height >= inner.y + inner.height
}

fn indexed_node_frame(node: &TemplatePaneNodeData, origin: &FrameRect) -> Option<FrameRect> {
    let mut frame = FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    if node.has_clip_frame {
        frame = intersect_frames(
            &frame,
            &FrameRect {
                x: origin.x + node.clip_frame.x,
                y: origin.y + node.clip_frame.y,
                width: node.clip_frame.width,
                height: node.clip_frame.height,
            },
        )?;
    }
    intersect_frames(&frame, origin)
}

fn intersect_frames(left: &FrameRect, right: &FrameRect) -> Option<FrameRect> {
    let x = left.x.max(right.x);
    let y = left.y.max(right.y);
    let right_edge = (left.x + left.width).min(right.x + right.width);
    let bottom_edge = (left.y + left.height).min(right.y + right.height);
    (right_edge > x && bottom_edge > y).then_some(FrameRect {
        x,
        y,
        width: right_edge - x,
        height: bottom_edge - y,
    })
}

fn cell_coordinate(value: f32) -> i32 {
    (value / HIT_INDEX_CELL_SIZE).floor() as i32
}
