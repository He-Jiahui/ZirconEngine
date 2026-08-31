use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[cfg(test)]
use std::cell::Cell;

use super::super::super::data::{
    FrameRect, HostDockPresentationPatch, HostWindowPresentationData, PaneData,
    TemplatePaneNodeData,
};
use super::super::super::template_geometry::template_nodes_bounds;
use super::surface_frame_builder::{accepts_pointer_move, is_dispatchable};
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

#[path = "index/persistent_buckets.rs"]
mod persistent_buckets;
use persistent_buckets::PersistentCellBuckets;
#[path = "index/geometry_patch.rs"]
mod geometry_patch;
use geometry_patch::{root_node_origin, same_frame_rect};
#[cfg(test)]
#[path = "index/incremental_tests.rs"]
mod incremental_tests;

const HIT_INDEX_CELL_SIZE: f32 = 64.0;
const PAINT_QUERY_SCRATCH_CAPACITY: usize = 256;
const EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID: &str = "WorkbenchExtensionModuleWorkspacesHost";

#[derive(Clone, Debug)]
pub(crate) struct HostExtensionWorkspacePaintIndex {
    pub(crate) root_row: usize,
    pub(crate) root_node_id: String,
    pub(crate) host_frame: FrameRect,
    host_row: usize,
}

pub(crate) struct HostWorkbenchHitIndex {
    indexed_nodes: ModelRc<TemplatePaneNodeData>,
    origin: Option<FrameRect>,
    origin_row: Option<usize>,
    buckets: Arc<PersistentCellBuckets>,
    paint_indices: Vec<HostTemplateNodePaintIndex>,
    popup_rows: Arc<Vec<usize>>,
    parent_rows: Arc<Vec<Option<usize>>>,
    extension_workspace: Option<HostExtensionWorkspacePaintIndex>,
    #[cfg(test)]
    last_candidate_visit_count: Cell<usize>,
    #[cfg(test)]
    query_count: Cell<usize>,
}

#[derive(Clone)]
struct HostTemplateNodePaintIndex {
    indexed_nodes: ModelRc<TemplatePaneNodeData>,
    origin: Option<FrameRect>,
    buckets: Arc<PersistentCellBuckets>,
    paint_order_rows: Arc<Vec<usize>>,
    query_scratch: Arc<Mutex<PaintRowQueryScratch>>,
    #[cfg(test)]
    query_sort_count: Cell<usize>,
}

struct PaintRowQueryScratch {
    seen: HashSet<usize>,
    rows: Vec<usize>,
    #[cfg(test)]
    allocation_count: usize,
}

impl PaintRowQueryScratch {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            seen: HashSet::with_capacity(capacity),
            rows: Vec::with_capacity(capacity),
            #[cfg(test)]
            allocation_count: 0,
        }
    }
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
        let origin_row = origin.as_ref().and_then(|origin| {
            nodes.iter().enumerate().find_map(|(row, node)| {
                root_node_origin(node)
                    .filter(|root_origin| same_frame_rect(root_origin, origin))
                    .map(|_| row)
            })
        });
        let mut index = Self {
            indexed_nodes: nodes.clone(),
            origin,
            origin_row,
            buckets: Arc::new(PersistentCellBuckets::default()),
            paint_indices: presentation_paint_node_models(presentation)
                .into_iter()
                .map(HostTemplateNodePaintIndex::new)
                .collect(),
            popup_rows: Arc::new(Vec::new()),
            parent_rows: Arc::new(parent_rows),
            extension_workspace,
            #[cfg(test)]
            last_candidate_visit_count: Cell::new(0),
            #[cfg(test)]
            query_count: Cell::new(0),
        };
        let Some(origin) = index.origin.clone() else {
            return index;
        };
        let mut buckets = HashMap::new();
        for (row, node) in nodes.iter().enumerate() {
            if node.popup_open && !node.disabled && !node.control_id.is_empty() {
                Arc::get_mut(&mut index.popup_rows)
                    .expect("new hit index popup rows must be uniquely owned")
                    .push(row);
            }
            let Some(frame) = indexed_node_frame(node, &origin) else {
                continue;
            };
            if accepts_pointer_move(node) {
                insert_frame(&mut buckets, row, &frame);
            }
        }
        for rows in buckets.values_mut() {
            sort_rows_in_paint_order(nodes, rows);
        }
        index.buckets = Arc::new(PersistentCellBuckets::from_cells(buckets));
        index
    }

    pub(crate) fn rebind_workbench_nodes(
        &self,
        previous_nodes: &ModelRc<TemplatePaneNodeData>,
        next_nodes: &ModelRc<TemplatePaneNodeData>,
        changed_rows: &[usize],
    ) -> Option<Self> {
        if !self.indexed_nodes.shares_values_with(previous_nodes)
            || previous_nodes.row_count() != next_nodes.row_count()
            || changed_rows.iter().any(|row| {
                previous_nodes
                    .get(*row)
                    .zip(next_nodes.get(*row))
                    .map_or(true, |(previous, next)| {
                        !same_index_membership(previous, next)
                    })
            })
        {
            return None;
        }
        let mut paint_indices = self.paint_indices.clone();
        let paint_index = paint_indices
            .iter_mut()
            .find(|index| index.indexed_nodes.shares_values_with(previous_nodes))?;
        paint_index.indexed_nodes = next_nodes.clone();
        Some(Self {
            indexed_nodes: next_nodes.clone(),
            origin: self.origin.clone(),
            origin_row: self.origin_row,
            buckets: Arc::clone(&self.buckets),
            paint_indices,
            popup_rows: Arc::clone(&self.popup_rows),
            parent_rows: Arc::clone(&self.parent_rows),
            extension_workspace: self.extension_workspace.clone(),
            #[cfg(test)]
            last_candidate_visit_count: Cell::new(0),
            #[cfg(test)]
            query_count: Cell::new(0),
        })
    }

    pub(crate) fn rebind_paint_models(
        &self,
        replacements: &[(ModelRc<TemplatePaneNodeData>, ModelRc<TemplatePaneNodeData>)],
    ) -> Option<Self> {
        let mut paint_indices = self.paint_indices.clone();
        for (previous_nodes, next_nodes) in replacements {
            if previous_nodes.shares_values_with(next_nodes) {
                continue;
            }
            if previous_nodes.row_count() == 0 || next_nodes.row_count() == 0 {
                return None;
            }
            let mut matches = paint_indices
                .iter()
                .enumerate()
                .filter(|(_, index)| index.indexed_nodes.shares_values_with(previous_nodes));
            let (position, _) = matches.next()?;
            if matches.next().is_some()
                || paint_indices.iter().enumerate().any(|(candidate, index)| {
                    candidate != position && index.indexed_nodes.shares_values_with(next_nodes)
                })
            {
                return None;
            }
            paint_indices[position] = HostTemplateNodePaintIndex::new(next_nodes.clone());
        }
        Some(Self {
            indexed_nodes: self.indexed_nodes.clone(),
            origin: self.origin.clone(),
            origin_row: self.origin_row,
            buckets: Arc::clone(&self.buckets),
            paint_indices,
            popup_rows: Arc::clone(&self.popup_rows),
            parent_rows: Arc::clone(&self.parent_rows),
            extension_workspace: self.extension_workspace.clone(),
            #[cfg(test)]
            last_candidate_visit_count: Cell::new(0),
            #[cfg(test)]
            query_count: Cell::new(0),
        })
    }

    pub(crate) fn rebind_presentation_dock_patch(
        &self,
        presentation: &HostWindowPresentationData,
        patch: &HostDockPresentationPatch,
        replacements: &[(ModelRc<TemplatePaneNodeData>, ModelRc<TemplatePaneNodeData>)],
    ) -> Option<Self> {
        let previous_models = presentation_dock_paint_node_models(presentation, patch);
        let next_models = dock_patch_paint_node_models(patch);
        if replacements.len() != previous_models.len()
            || replacements.len() != next_models.len()
            || replacements.iter().enumerate().any(|(index, pair)| {
                !pair.0.shares_values_with(&previous_models[index])
                    || !pair.1.shares_values_with(&next_models[index])
            })
        {
            return None;
        }
        self.rebind_paint_models(replacements)
    }

    pub(super) fn origin(&self) -> Option<&FrameRect> {
        self.origin.as_ref()
    }

    pub(crate) fn popup_rows(&self) -> &[usize] {
        &self.popup_rows
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
        self.indexes_paint_models(&models)
    }

    fn indexes_paint_models(&self, models: &[ModelRc<TemplatePaneNodeData>]) -> bool {
        models.len() == self.paint_indices.len()
            && models
                .iter()
                .zip(&self.paint_indices)
                .all(|(nodes, index)| index.indexed_nodes.shares_values_with(nodes))
    }

    pub(crate) fn extension_workspace(&self) -> Option<&HostExtensionWorkspacePaintIndex> {
        self.extension_workspace.as_ref()
    }

    #[cfg(test)]
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

    pub(crate) fn visit_paint_rows_for_nodes(
        &self,
        nodes: &ModelRc<TemplatePaneNodeData>,
        clip: &FrameRect,
        visit: &mut dyn FnMut(usize),
    ) -> bool {
        let Some(index) = self
            .paint_indices
            .iter()
            .find(|index| index.indexed_nodes.shares_values_with(nodes))
        else {
            return false;
        };
        record_current_ui_perf_counter(UiPerfCounter::WorkbenchPaintIndexQueryCount, 1.0);
        let candidate_count = index.visit_rows_for_clip(clip, visit);
        record_current_ui_perf_counter(
            UiPerfCounter::WorkbenchPaintIndexCandidateCount,
            candidate_count as f64,
        );
        true
    }

    pub(crate) fn visit_paint_rows_for_subtree(
        &self,
        root_row: usize,
        clip: &FrameRect,
        visit: &mut dyn FnMut(usize),
    ) -> bool {
        let Some(index) = self.paint_indices.first() else {
            return false;
        };
        record_current_ui_perf_counter(UiPerfCounter::WorkbenchPaintIndexQueryCount, 1.0);
        let candidate_count = index.visit_rows_for_clip(clip, &mut |row| {
            if reaches_ancestor(row, root_row, &self.parent_rows) {
                visit(row);
            }
        });
        record_current_ui_perf_counter(
            UiPerfCounter::WorkbenchPaintIndexCandidateCount,
            candidate_count as f64,
        );
        true
    }

    pub(super) fn candidate_rows(&self, x: f32, y: f32) -> &[usize] {
        self.buckets
            .get(&(cell_coordinate(x), cell_coordinate(y)))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(super) fn begin_query(&self) {
        #[cfg(test)]
        {
            self.last_candidate_visit_count.set(0);
            self.query_count
                .set(self.query_count.get().saturating_add(1));
        }
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

    #[cfg(test)]
    pub(crate) fn query_count_for_test(&self) -> usize {
        self.query_count.get()
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
            query_scratch: Arc::new(Mutex::new(PaintRowQueryScratch::with_capacity(
                indexed_nodes.row_count().min(PAINT_QUERY_SCRATCH_CAPACITY),
            ))),
            indexed_nodes,
            origin,
            buckets: Arc::new(PersistentCellBuckets::default()),
            paint_order_rows: Arc::new(Vec::new()),
            #[cfg(test)]
            query_sort_count: Cell::new(0),
        };
        let Some(origin) = index.origin.clone() else {
            return index;
        };
        let mut buckets = HashMap::new();
        for (row, node) in index.indexed_nodes.iter().enumerate() {
            if let Some(frame) = indexed_node_frame(node, &origin) {
                insert_frame(&mut buckets, row, &frame);
            }
        }
        let mut paint_order_rows = (0..index.indexed_nodes.row_count()).collect::<Vec<_>>();
        sort_rows_in_paint_order(&index.indexed_nodes, &mut paint_order_rows);
        let indexed_nodes = index.indexed_nodes.clone();
        for rows in buckets.values_mut() {
            sort_rows_in_paint_order(&indexed_nodes, rows);
        }
        index.buckets = Arc::new(PersistentCellBuckets::from_cells(buckets));
        index.paint_order_rows = Arc::new(paint_order_rows);
        index
    }

    #[cfg(test)]
    fn rows_for_clip(&self, clip: &FrameRect) -> Vec<usize> {
        let mut rows = Vec::new();
        self.visit_rows_for_clip(clip, &mut |row| rows.push(row));
        rows
    }

    fn visit_rows_for_clip(&self, clip: &FrameRect, visit: &mut dyn FnMut(usize)) -> usize {
        let Some(origin) = self.origin.as_ref() else {
            return 0;
        };
        let Some(clip) = intersect_frames(clip, origin) else {
            return 0;
        };
        if frame_contains(&clip, origin) {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.paint_index.full_order_reuse_count",
                1_u8
            );
            return visit_rows(self.paint_order_rows.as_slice(), visit);
        }

        let min_x = cell_coordinate(clip.x);
        let max_x = cell_coordinate(clip.x + clip.width - f32::EPSILON);
        let min_y = cell_coordinate(clip.y);
        let max_y = cell_coordinate(clip.y + clip.height - f32::EPSILON);
        if min_x == max_x && min_y == max_y {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.paint_index.single_cell_order_reuse_count",
                1_u8
            );
            let rows = self
                .buckets
                .get(&(min_x, min_y))
                .map(Vec::as_slice)
                .unwrap_or_default();
            return visit_rows(rows, visit);
        }
        zircon_runtime::profile_counter!("editor", "ui.paint_index.multi_cell_sort_count", 1_u8);
        let mut scratch = self
            .query_scratch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        scratch.seen.clear();
        scratch.rows.clear();
        let capacities_before = (scratch.seen.capacity(), scratch.rows.capacity());
        for cell_y in min_y..=max_y {
            for cell_x in min_x..=max_x {
                if let Some(bucket) = self.buckets.get(&(cell_x, cell_y)) {
                    for row in bucket {
                        if scratch.seen.insert(*row) {
                            scratch.rows.push(*row);
                        }
                    }
                }
            }
        }
        self.sort_rows_in_paint_order(&mut scratch.rows);
        if capacities_before != (scratch.seen.capacity(), scratch.rows.capacity()) {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.paint_index.query_scratch_growth_count",
                1_u8
            );
            #[cfg(test)]
            scratch.allocation_count = scratch.allocation_count.saturating_add(1);
        }
        visit_rows(scratch.rows.as_slice(), visit)
    }

    fn sort_rows_in_paint_order(&self, rows: &mut [usize]) {
        #[cfg(test)]
        self.query_sort_count
            .set(self.query_sort_count.get().saturating_add(1));
        sort_rows_in_paint_order(&self.indexed_nodes, rows);
    }

    #[cfg(test)]
    fn query_sort_count_for_test(&self) -> usize {
        self.query_sort_count.get()
    }

    #[cfg(test)]
    fn query_scratch_capacity_for_test(&self) -> (usize, usize) {
        let scratch = self
            .query_scratch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (scratch.seen.capacity(), scratch.rows.capacity())
    }

    #[cfg(test)]
    fn query_scratch_allocation_count_for_test(&self) -> usize {
        self.query_scratch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .allocation_count
    }
}

fn visit_rows(rows: &[usize], visit: &mut dyn FnMut(usize)) -> usize {
    for row in rows {
        visit(*row);
    }
    rows.len()
}

fn sort_rows_in_paint_order(indexed_nodes: &ModelRc<TemplatePaneNodeData>, rows: &mut [usize]) {
    rows.sort_unstable_by_key(|row| {
        (
            indexed_nodes
                .get(*row)
                .map(|node| node.z_index)
                .unwrap_or_default(),
            *row,
        )
    });
}

fn same_index_membership(previous: &TemplatePaneNodeData, next: &TemplatePaneNodeData) -> bool {
    previous.node_id == next.node_id
        && previous.parent_node_id == next.parent_node_id
        && previous.control_id == next.control_id
        && same_template_frame(&previous.frame, &next.frame)
        && previous.has_clip_frame == next.has_clip_frame
        && same_template_frame(&previous.clip_frame, &next.clip_frame)
        && previous.z_index == next.z_index
        && previous.popup_open == next.popup_open
        && previous.disabled == next.disabled
        && accepts_pointer_move(previous) == accepts_pointer_move(next)
        && !previous.control_id.starts_with("WorkbenchExtension")
        && !next.control_id.starts_with("WorkbenchExtension")
}

fn same_template_frame(
    previous: &super::super::super::data::TemplateNodeFrameData,
    next: &super::super::super::data::TemplateNodeFrameData,
) -> bool {
    previous.x == next.x
        && previous.y == next.y
        && previous.width == next.width
        && previous.height == next.height
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

fn presentation_dock_paint_node_models(
    presentation: &HostWindowPresentationData,
    patch: &HostDockPresentationPatch,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    match patch {
        HostDockPresentationPatch::Left(_) => {
            side_dock_paint_node_models(&presentation.host_scene_data.left_dock)
        }
        HostDockPresentationPatch::Right(_) => {
            side_dock_paint_node_models(&presentation.host_scene_data.right_dock)
        }
        HostDockPresentationPatch::Bottom(_) => {
            bottom_dock_paint_node_models(&presentation.host_scene_data.bottom_dock)
        }
    }
}

fn dock_patch_paint_node_models(
    patch: &HostDockPresentationPatch,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    match patch {
        HostDockPresentationPatch::Left(dock) | HostDockPresentationPatch::Right(dock) => {
            side_dock_paint_node_models(dock)
        }
        HostDockPresentationPatch::Bottom(dock) => bottom_dock_paint_node_models(dock),
    }
}

fn side_dock_paint_node_models(
    dock: &super::super::super::data::HostSideDockSurfaceData,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    let mut models = Vec::with_capacity(3);
    push_paint_model(&mut models, &dock.rail_nodes);
    push_paint_model(&mut models, &dock.header_nodes);
    push_pane_paint_model(&mut models, &dock.pane);
    models
}

fn bottom_dock_paint_node_models(
    dock: &super::super::super::data::HostBottomDockSurfaceData,
) -> Vec<ModelRc<TemplatePaneNodeData>> {
    let mut models = Vec::with_capacity(2);
    push_paint_model(&mut models, &dock.header_nodes);
    push_pane_paint_model(&mut models, &dock.pane);
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
        host_row: module_host_row,
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

#[cfg(test)]
mod tests {
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

        let move_hit =
            super::super::hit::hit_test_workbench_template_node_for_pointer_move_with_index(
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
        presentation.host_scene_data.left_dock.rail_nodes =
            model_rc(vec![paint_node("rail-old", 0.0)]);
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
}
