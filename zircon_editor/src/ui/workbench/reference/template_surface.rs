use std::collections::{BTreeSet, HashMap};

use thiserror::Error;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiSize},
    tree::UiTreeError,
};

use crate::ui::control::EditorUiControlService;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, RetainedUiHostNodeModel,
    RetainedUiHostProjection, RetainedUiProjection, RetainedUiProjectionSurfaceMetadataIndex,
    WORKBENCH_WINDOW_DOCUMENT_ID,
};

use super::EditorWorkbenchReferenceMetrics;

pub struct EditorWorkbenchTemplateControlIds;

impl EditorWorkbenchTemplateControlIds {
    pub const ROOT: &'static str = "WorkbenchWindowRoot";
    pub const TOP_TOOLBAR: &'static str = "WorkbenchWindowTopToolbarRegion";
    pub const MAIN_BAND: &'static str = "WorkbenchWindowMainBandRegion";
    pub const ACTIVITY_RAIL: &'static str = "WorkbenchMainBandActivityRail";
    pub const SCENE_TREE: &'static str = "WorkbenchMainBandSceneTreePanel";
    pub const VIEWPORT: &'static str = "WorkbenchMainBandViewportPanel";
    pub const VIEWPORT_TOOLBAR: &'static str = "WorkbenchViewportToolbar";
    pub const VIEWPORT_SURFACE: &'static str = "WorkbenchViewportSurface";
    pub const INSPECTOR: &'static str = "WorkbenchMainBandInspectorPanel";
    pub const COMPONENT_DRAWER: &'static str = "WorkbenchWindowComponentDrawerRegion";
    pub const STATUS_BAR: &'static str = "WorkbenchWindowStatusBarRegion";
    pub const PRIMARY_BUTTON: &'static str = "WorkbenchPrimaryButton";
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditorWorkbenchTemplateFrames {
    pub root: UiFrame,
    pub top_toolbar: UiFrame,
    pub main_band: UiFrame,
    pub activity_rail: UiFrame,
    pub scene_tree: UiFrame,
    pub viewport: UiFrame,
    pub inspector: UiFrame,
    pub component_drawer: UiFrame,
    pub status_bar: UiFrame,
}

impl EditorWorkbenchTemplateFrames {
    fn from_surface(
        surface: &UiSurface,
        control_nodes: &HashMap<String, UiNodeId>,
    ) -> Result<Self, EditorWorkbenchTemplateSurfaceError> {
        Ok(Self {
            root: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::ROOT,
            )?,
            top_toolbar: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::TOP_TOOLBAR,
            )?,
            main_band: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::MAIN_BAND,
            )?,
            activity_rail: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::ACTIVITY_RAIL,
            )?,
            scene_tree: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::SCENE_TREE,
            )?,
            viewport: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::VIEWPORT,
            )?,
            inspector: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::INSPECTOR,
            )?,
            component_drawer: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::COMPONENT_DRAWER,
            )?,
            status_bar: required_control_frame(
                surface,
                control_nodes,
                EditorWorkbenchTemplateControlIds::STATUS_BAR,
            )?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct EditorWorkbenchTemplateSurface {
    pub surface: UiSurface,
    pub metrics: EditorWorkbenchReferenceMetrics,
    pub frames: EditorWorkbenchTemplateFrames,
    pub host_projection: RetainedUiHostProjection,
    layout_size: UiSize,
    #[cfg(test)]
    layout_pass_count: u64,
    source_projection: RetainedUiProjection,
    source_projection_metadata_index: RetainedUiProjectionSurfaceMetadataIndex,
    control_nodes: HashMap<String, UiNodeId>,
    host_projection_node_indices: HashMap<UiNodeId, usize>,
    host_projection_topology: HashMap<UiNodeId, HostProjectionNodeIdentity>,
    host_projection_roots: Vec<UiNodeId>,
    pending_host_projection_patch_indices: BTreeSet<usize>,
    pending_host_projection_has_semantic_changes: bool,
    host_projection_full_refresh_pending: bool,
    #[cfg(test)]
    host_projection_full_rebuild_count: u64,
    #[cfg(test)]
    last_host_projection_patch_count: usize,
    #[cfg(test)]
    last_host_projection_semantic_patch_count: usize,
    #[cfg(test)]
    last_host_projection_geometry_patch_count: usize,
    #[cfg(test)]
    frames_extract_count: u64,
    #[cfg(test)]
    frames_extract_skip_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HostProjectionNodeIdentity {
    node_path: String,
    parent: Option<UiNodeId>,
    children: Vec<UiNodeId>,
    component: String,
    control_id: Option<String>,
}

#[derive(Clone, Debug, Default)]
struct HostProjectionRefreshWorkset {
    semantic_node_ids: BTreeSet<UiNodeId>,
    geometry_node_ids: BTreeSet<UiNodeId>,
}

impl HostProjectionRefreshWorkset {
    fn changed_node_ids(&self) -> BTreeSet<UiNodeId> {
        let mut node_ids = self.semantic_node_ids.clone();
        node_ids.extend(self.geometry_node_ids.iter().copied());
        node_ids
    }
}

#[derive(Clone, Copy, Debug)]
struct HostProjectionGeometryPatch {
    index: usize,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    z_index: i32,
}

impl EditorWorkbenchTemplateSurface {
    pub fn recompute_layout(
        &mut self,
        runtime: &EditorUiHostRuntime,
        size: UiSize,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        let semantic_node_ids = self.surface.pending_rebuild_node_ids();
        zircon_runtime::profile_counter!(
            "editor",
            "ui.workbench_template.pending_changed_node_count",
            self.surface.pending_invalidation_changed_node_count()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.workbench_template.tree_node_count",
            self.surface.tree.nodes.len()
        );
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "workbench_surface_compute_layout"
            );
            self.surface.compute_layout(size)?;
        }
        let workset = HostProjectionRefreshWorkset {
            semantic_node_ids,
            geometry_node_ids: self.surface.last_layout_geometry_changed_node_ids().clone(),
        };
        zircon_runtime::profile_counter!(
            "editor",
            "ui.workbench_template.geometry_changed_node_count",
            self.surface.last_layout_geometry_changed_node_ids().len()
        );
        zircon_runtime::profile_counter!("editor", "ui.workbench_template.layout_pass_count", 1);
        #[cfg(test)]
        {
            self.layout_pass_count = self.layout_pass_count.saturating_add(1);
        }
        self.layout_size = size;
        self.refresh_projection(runtime, &workset, true)
    }

    pub(crate) fn refresh_after_state_change(
        &mut self,
        runtime: &EditorUiHostRuntime,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        let semantic_node_ids = self.surface.pending_rebuild_node_ids();
        let report = self.surface.rebuild_dirty(self.layout_size)?;
        let workset = HostProjectionRefreshWorkset {
            semantic_node_ids,
            geometry_node_ids: self.surface.last_layout_geometry_changed_node_ids().clone(),
        };
        if report.layout_recomputed {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_template.layout_pass_count",
                1
            );
            #[cfg(test)]
            {
                self.layout_pass_count = self.layout_pass_count.saturating_add(1);
            }
        }
        // Pointer feedback and other render/input-only mutations leave the control geometry
        // authoritative. Reuse the previously extracted frame snapshot until layout runs.
        self.refresh_projection(runtime, &workset, report.layout_recomputed)
    }

    #[cfg(test)]
    pub(crate) fn layout_pass_count(&self) -> u64 {
        self.layout_pass_count
    }

    #[cfg(test)]
    pub(crate) fn frames_extract_count(&self) -> u64 {
        self.frames_extract_count
    }

    #[cfg(test)]
    pub(crate) fn frames_extract_skip_count(&self) -> u64 {
        self.frames_extract_skip_count
    }

    pub fn control_frame(&self, control_id: &str) -> Option<UiFrame> {
        let node_id = self.control_node_id(control_id)?;
        self.surface
            .tree
            .nodes
            .get(&node_id)
            .map(|node| node.layout_cache.frame)
    }

    pub fn visible_control_frame(&self, control_id: &str) -> Option<UiFrame> {
        visible_arranged_control_frame(&self.surface, self.control_node_id(control_id)?)
    }

    pub(crate) fn host_projection_node_for_control(
        &self,
        control_id: &str,
    ) -> Option<&RetainedUiHostNodeModel> {
        let node_id = self.control_node_id(control_id)?;
        let index = self.host_projection_node_indices.get(&node_id)?;
        self.host_projection.nodes.get(*index)
    }

    fn refresh_projection(
        &mut self,
        runtime: &EditorUiHostRuntime,
        workset: &HostProjectionRefreshWorkset,
        refresh_frames: bool,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        if refresh_frames {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "workbench_surface_extract_frames"
            );
            self.frames =
                EditorWorkbenchTemplateFrames::from_surface(&self.surface, &self.control_nodes)?;
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_template.frames_extract_count",
                1
            );
            #[cfg(test)]
            {
                self.frames_extract_count = self.frames_extract_count.saturating_add(1);
            }
        } else {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_template.frames_extract_skip_count",
                1
            );
            #[cfg(test)]
            {
                self.frames_extract_skip_count = self.frames_extract_skip_count.saturating_add(1);
            }
        }
        {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "workbench_surface_build_host_projection"
            );
            let changed_node_ids = workset.changed_node_ids();
            if changed_node_ids.is_empty() && self.host_projection_roots == self.surface.tree.roots
            {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.workbench_template.host_projection_noop_count",
                    1
                );
                #[cfg(test)]
                {
                    self.last_host_projection_patch_count = 0;
                    self.last_host_projection_semantic_patch_count = 0;
                    self.last_host_projection_geometry_patch_count = 0;
                }
                return Ok(());
            }
            if !self.can_patch_host_projection(&changed_node_ids) {
                return self.refresh_projection_full(runtime);
            }
            let patched_nodes = runtime.build_retained_host_nodes_with_surface(
                &self.source_projection,
                &self.surface,
                &workset.semantic_node_ids,
                &self.source_projection_metadata_index,
            )?;
            if patched_nodes.len() != workset.semantic_node_ids.len() {
                return self.refresh_projection_full(runtime);
            }
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_template.host_projection_metadata_lookup_count",
                patched_nodes.len()
            );
            let mut projection_patches = Vec::with_capacity(patched_nodes.len());
            for (node_id, node) in patched_nodes {
                let Some(index) = self.host_projection_node_indices.get(&node_id).copied() else {
                    return self.refresh_projection_full(runtime);
                };
                projection_patches.push((index, node));
            }
            let mut geometry_patches = Vec::with_capacity(workset.geometry_node_ids.len());
            for node_id in workset
                .geometry_node_ids
                .difference(&workset.semantic_node_ids)
            {
                let Some(index) = self.host_projection_node_indices.get(node_id).copied() else {
                    return self.refresh_projection_full(runtime);
                };
                let Some((frame, clip_frame, z_index)) =
                    surface_host_geometry(&self.surface, *node_id)
                else {
                    return self.refresh_projection_full(runtime);
                };
                geometry_patches.push(HostProjectionGeometryPatch {
                    index,
                    frame,
                    clip_frame,
                    z_index,
                });
            }
            for (index, node) in projection_patches {
                self.host_projection.nodes[index] = node;
                self.pending_host_projection_patch_indices.insert(index);
            }
            self.pending_host_projection_has_semantic_changes |=
                !workset.semantic_node_ids.is_empty();
            for patch in &geometry_patches {
                let node = &mut self.host_projection.nodes[patch.index];
                node.frame = patch.frame;
                node.clip_frame = patch.clip_frame;
                node.z_index = patch.z_index;
                self.pending_host_projection_patch_indices
                    .insert(patch.index);
            }
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_template.host_projection_patched_node_count",
                changed_node_ids.len()
            );
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_template.host_projection_semantic_patch_count",
                workset.semantic_node_ids.len()
            );
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_template.host_projection_geometry_patch_count",
                geometry_patches.len()
            );
            zircon_runtime::profile_counter!(
                "editor",
                "ui.workbench_template.host_projection_incremental_count",
                1
            );
            #[cfg(test)]
            {
                self.last_host_projection_patch_count = changed_node_ids.len();
                self.last_host_projection_semantic_patch_count = workset.semantic_node_ids.len();
                self.last_host_projection_geometry_patch_count = geometry_patches.len();
            }
        }
        Ok(())
    }

    fn can_patch_host_projection(&self, changed_node_ids: &BTreeSet<UiNodeId>) -> bool {
        if self.host_projection_roots != self.surface.tree.roots {
            return false;
        }
        changed_node_ids.iter().all(|node_id| {
            let Some(node) = self.surface.tree.nodes.get(node_id) else {
                return false;
            };
            let Some(previous_identity) = self.host_projection_topology.get(node_id) else {
                return false;
            };
            if !previous_identity.matches_surface_node(node) {
                return false;
            }
            self.host_projection_node_indices
                .get(node_id)
                .and_then(|index| self.host_projection.nodes.get(*index))
                .is_some_and(|host_node| host_node.node_id == node.node_path.0)
        })
    }

    fn refresh_projection_full(
        &mut self,
        runtime: &EditorUiHostRuntime,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        self.host_projection = runtime
            .build_retained_host_projection_with_surface(&self.source_projection, &self.surface)?;
        let (indices, topology) = build_host_projection_state(&self.surface, &self.host_projection);
        self.host_projection_node_indices = indices;
        self.host_projection_topology = topology;
        self.host_projection_roots = self.surface.tree.roots.clone();
        self.pending_host_projection_patch_indices.clear();
        self.pending_host_projection_has_semantic_changes = false;
        self.host_projection_full_refresh_pending = true;
        zircon_runtime::profile_counter!(
            "editor",
            "ui.workbench_template.host_projection_full_rebuild_count",
            1
        );
        #[cfg(test)]
        {
            self.host_projection_full_rebuild_count =
                self.host_projection_full_rebuild_count.saturating_add(1);
            self.last_host_projection_patch_count = 0;
            self.last_host_projection_semantic_patch_count = 0;
            self.last_host_projection_geometry_patch_count = 0;
        }
        Ok(())
    }

    pub(crate) fn control_node_id(&self, control_id: &str) -> Option<UiNodeId> {
        self.control_nodes.get(control_id).copied()
    }

    #[cfg(test)]
    pub(crate) fn has_control_index_entry(&self, control_id: &str) -> bool {
        self.control_nodes.contains_key(control_id)
    }

    /// Rebuilds the lookup only after retained row topology changes. Sparse property patches
    /// rely on this map remaining authoritative and must not fall back to scanning the surface.
    pub(crate) fn refresh_control_node_index(
        &mut self,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        self.control_nodes = build_control_node_index(&self.surface)?;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn last_host_projection_patch_count(&self) -> usize {
        self.last_host_projection_patch_count
    }

    #[cfg(test)]
    pub(crate) fn host_projection_full_rebuild_count(&self) -> u64 {
        self.host_projection_full_rebuild_count
    }

    #[cfg(test)]
    pub(crate) fn last_host_projection_semantic_patch_count(&self) -> usize {
        self.last_host_projection_semantic_patch_count
    }

    #[cfg(test)]
    pub(crate) fn last_host_projection_geometry_patch_count(&self) -> usize {
        self.last_host_projection_geometry_patch_count
    }

    #[cfg(test)]
    pub(crate) fn clear_host_projection_index_for_test(&mut self) {
        self.host_projection_node_indices.clear();
    }

    #[cfg(test)]
    pub(crate) fn full_host_projection_for_test(
        &self,
        runtime: &EditorUiHostRuntime,
    ) -> Result<RetainedUiHostProjection, EditorUiHostRuntimeError> {
        runtime.build_retained_host_projection_with_surface(&self.source_projection, &self.surface)
    }

    pub(crate) fn pending_host_projection_patch_nodes(
        &self,
    ) -> Option<Vec<RetainedUiHostNodeModel>> {
        if self.host_projection_full_refresh_pending {
            return None;
        }
        Some(
            self.pending_host_projection_patch_indices
                .iter()
                .filter_map(|index| self.host_projection.nodes.get(*index).cloned())
                .collect(),
        )
    }

    pub(crate) fn pending_host_projection_geometry_patch_indices(&self) -> Option<Vec<usize>> {
        if self.host_projection_full_refresh_pending
            || self.pending_host_projection_has_semantic_changes
        {
            return None;
        }
        Some(
            self.pending_host_projection_patch_indices
                .iter()
                .copied()
                .collect(),
        )
    }

    pub(crate) fn has_pending_host_projection_commit(&self) -> bool {
        self.host_projection_full_refresh_pending
            || !self.pending_host_projection_patch_indices.is_empty()
    }

    pub(crate) fn mark_host_projection_committed(&mut self) {
        self.pending_host_projection_patch_indices.clear();
        self.pending_host_projection_has_semantic_changes = false;
        self.host_projection_full_refresh_pending = false;
    }
}

impl HostProjectionNodeIdentity {
    fn matches_surface_node(&self, node: &zircon_runtime_interface::ui::tree::UiTreeNode) -> bool {
        let (component, control_id) = node
            .template_metadata
            .as_ref()
            .map(|metadata| (metadata.component.as_str(), metadata.control_id.as_deref()))
            .unwrap_or_default();
        self.node_path == node.node_path.0
            && self.parent == node.parent
            && self.children == node.children
            && self.component == component
            && self.control_id.as_deref() == control_id
    }

    fn from_surface_node(node: &zircon_runtime_interface::ui::tree::UiTreeNode) -> Self {
        let (component, control_id) = node
            .template_metadata
            .as_ref()
            .map(|metadata| (metadata.component.clone(), metadata.control_id.clone()))
            .unwrap_or_default();
        Self {
            node_path: node.node_path.0.clone(),
            parent: node.parent,
            children: node.children.clone(),
            component,
            control_id,
        }
    }
}

#[derive(Debug, Error)]
pub enum EditorWorkbenchTemplateSurfaceError {
    #[error(transparent)]
    Runtime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
    #[error("componentized workbench template is missing required control {control_id}")]
    MissingControl { control_id: &'static str },
    #[error(
        "componentized workbench template control {control_id} is duplicated at {first:?} and {duplicate:?}"
    )]
    DuplicateControl {
        control_id: String,
        first: UiNodeId,
        duplicate: UiNodeId,
    },
}

pub fn build_editor_workbench_template_surface(
    runtime: &EditorUiHostRuntime,
    metrics: EditorWorkbenchReferenceMetrics,
) -> Result<EditorWorkbenchTemplateSurface, EditorWorkbenchTemplateSurfaceError> {
    let mut source_projection = runtime.project_document(WORKBENCH_WINDOW_DOCUMENT_ID)?;
    let mut route_service = EditorUiControlService::default();
    runtime.register_projection_routes(&mut route_service, &mut source_projection)?;
    let source_projection_metadata_index = source_projection.surface_metadata_index();
    let mut surface = runtime.build_shared_surface(WORKBENCH_WINDOW_DOCUMENT_ID)?;
    surface.compute_layout(metrics.target_size())?;
    let control_nodes = build_control_node_index(&surface)?;
    let frames = EditorWorkbenchTemplateFrames::from_surface(&surface, &control_nodes)?;
    let host_projection =
        runtime.build_retained_host_projection_with_surface(&source_projection, &surface)?;
    let (host_projection_node_indices, host_projection_topology) =
        build_host_projection_state(&surface, &host_projection);
    let host_projection_roots = surface.tree.roots.clone();
    Ok(EditorWorkbenchTemplateSurface {
        surface,
        metrics,
        frames,
        host_projection,
        layout_size: metrics.target_size(),
        #[cfg(test)]
        layout_pass_count: 1,
        source_projection,
        source_projection_metadata_index,
        control_nodes,
        host_projection_node_indices,
        host_projection_topology,
        host_projection_roots,
        pending_host_projection_patch_indices: BTreeSet::new(),
        pending_host_projection_has_semantic_changes: false,
        host_projection_full_refresh_pending: true,
        #[cfg(test)]
        host_projection_full_rebuild_count: 1,
        #[cfg(test)]
        last_host_projection_patch_count: 0,
        #[cfg(test)]
        last_host_projection_semantic_patch_count: 0,
        #[cfg(test)]
        last_host_projection_geometry_patch_count: 0,
        #[cfg(test)]
        frames_extract_count: 1,
        #[cfg(test)]
        frames_extract_skip_count: 0,
    })
}

fn surface_host_geometry(
    surface: &UiSurface,
    node_id: UiNodeId,
) -> Option<(UiFrame, Option<UiFrame>, i32)> {
    let node = surface.tree.nodes.get(&node_id)?;
    Some(match surface.arranged_node(node_id) {
        Some(arranged) => (arranged.frame, Some(arranged.clip_frame), arranged.z_index),
        None => (
            node.layout_cache.frame,
            node.layout_cache.clip_frame,
            node.z_index,
        ),
    })
}

fn build_host_projection_state(
    surface: &UiSurface,
    host_projection: &RetainedUiHostProjection,
) -> (
    HashMap<UiNodeId, usize>,
    HashMap<UiNodeId, HostProjectionNodeIdentity>,
) {
    let projection_indices_by_path = host_projection
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.node_id.as_str(), index))
        .collect::<HashMap<_, _>>();
    let mut node_indices = HashMap::with_capacity(surface.tree.nodes.len());
    let mut topology = HashMap::with_capacity(surface.tree.nodes.len());
    for (node_id, node) in &surface.tree.nodes {
        if let Some(index) = projection_indices_by_path.get(node.node_path.0.as_str()) {
            node_indices.insert(*node_id, *index);
        }
        topology.insert(
            *node_id,
            HostProjectionNodeIdentity::from_surface_node(node),
        );
    }
    (node_indices, topology)
}

fn required_control_frame(
    surface: &UiSurface,
    control_nodes: &HashMap<String, UiNodeId>,
    control_id: &'static str,
) -> Result<UiFrame, EditorWorkbenchTemplateSurfaceError> {
    control_nodes
        .get(control_id)
        .and_then(|node_id| surface.tree.nodes.get(node_id))
        .map(|node| node.layout_cache.frame)
        .ok_or(EditorWorkbenchTemplateSurfaceError::MissingControl { control_id })
}

fn build_control_node_index(
    surface: &UiSurface,
) -> Result<HashMap<String, UiNodeId>, EditorWorkbenchTemplateSurfaceError> {
    let mut control_nodes = HashMap::new();
    for (node_id, node) in &surface.tree.nodes {
        let Some(control_id) = node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
        else {
            continue;
        };
        if let Some(first) = control_nodes.insert(control_id.to_string(), *node_id) {
            return Err(EditorWorkbenchTemplateSurfaceError::DuplicateControl {
                control_id: control_id.to_string(),
                first,
                duplicate: *node_id,
            });
        }
    }
    Ok(control_nodes)
}

fn visible_arranged_control_frame(surface: &UiSurface, node_id: UiNodeId) -> Option<UiFrame> {
    let node = surface.arranged_node(node_id)?;
    if !surface_node_render_visible(surface, node_id) {
        return None;
    }
    node.frame.intersection(node.clip_frame)
}

fn surface_node_render_visible(surface: &UiSurface, node_id: UiNodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        if let Some(node) = surface.arranged_node(id) {
            if !node.is_render_visible() {
                return false;
            }
            current = node.parent;
        } else if let Some(node) = surface.tree.nodes.get(&id) {
            if !node.is_render_visible() {
                return false;
            }
            current = node.parent;
        } else {
            return false;
        };
    }
    true
}
