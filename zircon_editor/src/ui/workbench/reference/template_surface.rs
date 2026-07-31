use std::collections::HashMap;

use thiserror::Error;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiSize},
    tree::UiTreeError,
};

use crate::ui::control::EditorUiControlService;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, RetainedUiHostProjection, RetainedUiProjection,
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
    source_projection: RetainedUiProjection,
    control_nodes: HashMap<String, UiNodeId>,
}

impl EditorWorkbenchTemplateSurface {
    pub fn recompute_layout(
        &mut self,
        runtime: &EditorUiHostRuntime,
        size: UiSize,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        self.surface.compute_layout(size)?;
        self.layout_size = size;
        self.refresh_projection(runtime)
    }

    pub(crate) fn refresh_after_state_change(
        &mut self,
        runtime: &EditorUiHostRuntime,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        self.surface.rebuild_dirty(self.layout_size)?;
        self.refresh_projection(runtime)
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

    fn refresh_projection(
        &mut self,
        runtime: &EditorUiHostRuntime,
    ) -> Result<(), EditorWorkbenchTemplateSurfaceError> {
        self.frames =
            EditorWorkbenchTemplateFrames::from_surface(&self.surface, &self.control_nodes)?;
        self.host_projection = runtime
            .build_retained_host_projection_with_surface(&self.source_projection, &self.surface)?;
        Ok(())
    }

    fn control_node_id(&self, control_id: &str) -> Option<UiNodeId> {
        self.control_nodes
            .get(control_id)
            .copied()
            .or_else(|| find_control_node_id(&self.surface, control_id))
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
}

pub fn build_editor_workbench_template_surface(
    runtime: &EditorUiHostRuntime,
    metrics: EditorWorkbenchReferenceMetrics,
) -> Result<EditorWorkbenchTemplateSurface, EditorWorkbenchTemplateSurfaceError> {
    let mut source_projection = runtime.project_document(WORKBENCH_WINDOW_DOCUMENT_ID)?;
    let mut route_service = EditorUiControlService::default();
    runtime.register_projection_routes(&mut route_service, &mut source_projection)?;
    let mut surface = runtime.build_shared_surface(WORKBENCH_WINDOW_DOCUMENT_ID)?;
    surface.compute_layout(metrics.target_size())?;
    let control_nodes = build_control_node_index(&surface);
    let frames = EditorWorkbenchTemplateFrames::from_surface(&surface, &control_nodes)?;
    let host_projection =
        runtime.build_retained_host_projection_with_surface(&source_projection, &surface)?;
    Ok(EditorWorkbenchTemplateSurface {
        surface,
        metrics,
        frames,
        host_projection,
        layout_size: metrics.target_size(),
        source_projection,
        control_nodes,
    })
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

fn build_control_node_index(surface: &UiSurface) -> HashMap<String, UiNodeId> {
    let mut control_nodes = HashMap::new();
    for (node_id, node) in &surface.tree.nodes {
        let Some(control_id) = node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
        else {
            continue;
        };
        control_nodes
            .entry(control_id.to_string())
            .or_insert(*node_id);
    }
    control_nodes
}

fn find_control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    surface.tree.nodes.iter().find_map(|(node_id, node)| {
        (node
            .template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some(control_id))
        .then_some(*node_id)
    })
}

fn visible_arranged_control_frame(surface: &UiSurface, node_id: UiNodeId) -> Option<UiFrame> {
    let node = surface.arranged_tree.get(node_id)?;
    if !surface_node_render_visible(surface, node_id) {
        return None;
    }
    node.frame.intersection(node.clip_frame)
}

fn surface_node_render_visible(surface: &UiSurface, node_id: UiNodeId) -> bool {
    let mut current = Some(node_id);
    while let Some(id) = current {
        if let Some(node) = surface.arranged_tree.get(id) {
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
