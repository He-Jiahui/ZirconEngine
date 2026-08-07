use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::super::super::super::data::{
    paint_pane_interaction_state, paint_text_input_focus, paint_viewport_image,
    paint_workbench_hit_index, FrameRect, HostWindowLayoutData, HostWindowPresentationData,
    TemplatePaneNodeData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_template_nodes::{
    draw_template_nodes, draw_template_nodes_with_transform, has_template_nodes,
    is_viewport_fallback_scene_node, TemplateNodePaintTransform,
};
use super::super::super::super::surface_hit_test::HostWorkbenchHitIndex;
use super::super::super::root_frames::{resolve_root_frames, zero_origin};
use super::super::{chrome, dock_layer, resize};
use super::modal;
use super::page_overflow::draw_host_page_overflow_menu;
use super::root_template::{draw_root_template_overlay, frame_bounds};
use crate::ui::retained_host::primitives::ModelRc;

const EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID: &str = "WorkbenchExtensionModuleWorkspacesHost";

struct ComponentizedChromeFallbackTransform {
    suppress_viewport_fallback: bool,
}

impl ComponentizedChromeFallbackTransform {
    fn from_presentation(presentation: &HostWindowPresentationData) -> Self {
        let viewport_image = paint_viewport_image(presentation);
        Self {
            suppress_viewport_fallback: viewport_image
                .as_ref()
                .is_some_and(|image| image.is_valid()),
        }
    }
}

impl TemplateNodePaintTransform for ComponentizedChromeFallbackTransform {
    fn transform(
        &self,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        if self.suppress_viewport_fallback && is_viewport_fallback_scene_node(&node) {
            None
        } else {
            Some((node, clip))
        }
    }
}

pub(in crate::ui::retained_host::host_contract) fn draws_componentized_workbench_window(
    presentation: &HostWindowPresentationData,
) -> bool {
    has_template_nodes(&presentation.workbench_window_nodes)
}

pub(in crate::ui::retained_host::host_contract) fn draw_componentized_workbench_window(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
) {
    let pane_interaction_state = paint_pane_interaction_state(presentation);
    frame.set_pane_interaction_state(&pane_interaction_state);
    let frame_bounds = frame_bounds(frame);
    let root = resolve_root_frames(frame.width(), frame.height(), presentation);
    chrome::draw_top_chrome_layers(frame, &root, presentation);
    draw_componentized_workbench_chrome(frame, presentation, &frame_bounds);
    // The template owns its mounted toolbar/status chrome and activated extension workspace.
    // Host scene data remains authoritative for outer menu/page chrome, ordinary panes,
    // viewport, splitters, and floating surfaces.
    dock_layer::draw_dock_layers(frame, presentation);
    draw_componentized_extension_workspace(frame, presentation, &frame_bounds);
    resize::draw_resize_layer(frame, presentation);
    dock_layer::draw_floating_layer(frame, presentation);
    draw_host_page_overflow_menu(frame, presentation);
    modal::draw_menu_and_prompt_layers(frame, presentation);
    draw_root_template_overlay(frame, presentation);
}

fn draw_componentized_extension_workspace(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    frame_bounds: &FrameRect,
) -> bool {
    let paint_index = paint_workbench_hit_index(&presentation.workbench_window_nodes);
    let Some(workspace_region) = componentized_extension_workspace_region(
        presentation,
        frame_bounds,
        paint_index.as_deref(),
    ) else {
        return false;
    };
    if frame
        .paint_clip()
        .is_some_and(|damage| intersect_rect(&workspace_region.clip, damage).is_none())
    {
        return false;
    }
    let subtree = ExtensionWorkspaceSubtree::from_presentation(
        presentation,
        workspace_region.root_node_id.as_str(),
        workspace_region.root_row,
        paint_index,
    );
    let text_input_focus = paint_text_input_focus(presentation);

    draw_template_nodes_with_transform(
        frame,
        &presentation.workbench_window_nodes,
        &zero_origin(),
        &workspace_region.clip,
        Some(&text_input_focus),
        Some(&subtree),
    )
}

#[cfg(test)]
pub(crate) fn paint_componentized_extension_workspace_for_test(
    width: u32,
    height: u32,
    background: [u8; 4],
    presentation: &HostWindowPresentationData,
) -> Vec<u8> {
    let mut frame = HostRgbaFrame::filled(width, height, background);
    let frame_bounds = frame_bounds(&frame);
    draw_componentized_extension_workspace(&mut frame, presentation, &frame_bounds);
    frame.into_bytes()
}

struct ExtensionWorkspacePaintRegion {
    root_node_id: String,
    root_row: Option<usize>,
    clip: FrameRect,
}

struct ExtensionWorkspaceSubtree {
    indexed_root: Option<(Arc<HostWorkbenchHitIndex>, usize)>,
    included_rows: Vec<usize>,
}

impl ExtensionWorkspaceSubtree {
    fn from_presentation(
        presentation: &HostWindowPresentationData,
        root_node_id: &str,
        root_row: Option<usize>,
        paint_index: Option<Arc<HostWorkbenchHitIndex>>,
    ) -> Self {
        if let (Some(root_row), Some(index)) = (root_row, paint_index) {
            return Self {
                indexed_root: Some((index, root_row)),
                included_rows: Vec::new(),
            };
        }
        let nodes = &presentation.workbench_window_nodes;
        let nodes_by_node_id = node_index(nodes);
        let included_rows = nodes
            .iter()
            .enumerate()
            .filter_map(|(row, node)| {
                reaches_subtree_root(node.node_id.as_str(), root_node_id, &nodes_by_node_id)
                    .then_some(row)
            })
            .collect();
        Self {
            indexed_root: None,
            included_rows,
        }
    }
}

impl TemplateNodePaintTransform for ExtensionWorkspaceSubtree {
    fn row_visit_indices(&self, _row_count: usize, clip: &FrameRect) -> Option<Vec<usize>> {
        Some(
            self.indexed_root
                .as_ref()
                .map(|(index, root_row)| index.paint_rows_for_subtree(*root_row, clip))
                .unwrap_or_else(|| self.included_rows.clone()),
        )
    }

    fn transform(
        &self,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        Some((node, clip))
    }
}

fn reaches_subtree_root<'a>(
    node_id: &'a str,
    root_node_id: &str,
    nodes_by_node_id: &HashMap<&'a str, &'a TemplatePaneNodeData>,
) -> bool {
    let mut current_node_id = node_id;
    let mut visited_node_ids = HashSet::new();
    loop {
        if current_node_id == root_node_id {
            return true;
        }
        if !visited_node_ids.insert(current_node_id) {
            return false;
        }
        let Some(node) = nodes_by_node_id.get(current_node_id) else {
            return false;
        };
        if node.parent_node_id.is_empty() {
            return false;
        }
        current_node_id = node.parent_node_id.as_str();
    }
}

fn componentized_extension_workspace_region(
    presentation: &HostWindowPresentationData,
    frame_bounds: &FrameRect,
    paint_index: Option<&HostWorkbenchHitIndex>,
) -> Option<ExtensionWorkspacePaintRegion> {
    if let Some(workspace) = paint_index.and_then(HostWorkbenchHitIndex::extension_workspace) {
        return Some(ExtensionWorkspacePaintRegion {
            root_node_id: workspace.root_node_id.clone(),
            root_row: Some(workspace.root_row),
            clip: intersect_rect(&workspace.host_frame, frame_bounds)?,
        });
    }
    let nodes = &presentation.workbench_window_nodes;
    let module_workspaces_host = nodes
        .iter()
        .find(|node| node.control_id.as_str() == EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID)?;
    let active_workspace_root_node_id =
        active_extension_workspace_root_node_id(&nodes, module_workspaces_host.node_id.as_str())?;
    let workspace_frame = FrameRect {
        x: module_workspaces_host.frame.x,
        y: module_workspaces_host.frame.y,
        width: module_workspaces_host.frame.width,
        height: module_workspaces_host.frame.height,
    };
    Some(ExtensionWorkspacePaintRegion {
        root_node_id: active_workspace_root_node_id,
        root_row: None,
        clip: intersect_rect(&workspace_frame, frame_bounds)?,
    })
}

fn active_extension_workspace_root_node_id(
    nodes: &ModelRc<TemplatePaneNodeData>,
    module_workspaces_host_node_id: &str,
) -> Option<String> {
    let nodes_by_node_id = node_index(nodes);

    nodes
        .iter()
        .find(|node| {
            is_extension_workspace_root_control(node.control_id.as_str())
                && nodes_by_node_id
                    .get(node.parent_node_id.as_str())
                    .is_some_and(|parent| {
                        is_extension_workspace_host_control(parent.control_id.as_str())
                    })
                && reaches_subtree_root(
                    node.node_id.as_str(),
                    module_workspaces_host_node_id,
                    &nodes_by_node_id,
                )
        })
        .map(|node| node.node_id.as_str().to_string())
}

fn node_index(nodes: &ModelRc<TemplatePaneNodeData>) -> HashMap<&str, &TemplatePaneNodeData> {
    nodes
        .iter()
        .filter(|node| !node.node_id.is_empty())
        .map(|node| (node.node_id.as_str(), node))
        .collect()
}

fn is_extension_workspace_root_control(control_id: &str) -> bool {
    control_id.starts_with("WorkbenchExtension") && control_id.ends_with("Workspace")
}

fn is_extension_workspace_host_control(control_id: &str) -> bool {
    control_id.starts_with("WorkbenchExtension") && control_id.ends_with("WorkspaceHost")
}

fn intersect_rect(rect: &FrameRect, bounds: &FrameRect) -> Option<FrameRect> {
    let left = rect.x.max(bounds.x);
    let top = rect.y.max(bounds.y);
    let right = (rect.x + rect.width).min(bounds.x + bounds.width);
    let bottom = (rect.y + rect.height).min(bounds.y + bounds.height);
    (right > left && bottom > top).then_some(FrameRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

fn draw_componentized_workbench_chrome(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    frame_bounds: &FrameRect,
) {
    let Some((top_chrome, status_bar)) =
        componentized_chrome_clips(&presentation.host_layout, frame_bounds)
    else {
        let transform = ComponentizedChromeFallbackTransform::from_presentation(presentation);
        let text_input_focus = paint_text_input_focus(presentation);
        draw_template_nodes_with_transform(
            frame,
            &presentation.workbench_window_nodes,
            &zero_origin(),
            frame_bounds,
            Some(&text_input_focus),
            Some(&transform),
        );
        return;
    };

    draw_componentized_workbench_chrome_clip(frame, presentation, &top_chrome);
    draw_componentized_workbench_chrome_clip(frame, presentation, &status_bar);
}

fn draw_componentized_workbench_chrome_clip(
    frame: &mut HostRgbaFrame,
    presentation: &HostWindowPresentationData,
    clip: &FrameRect,
) {
    if !visible_rect(clip) {
        return;
    }
    let text_input_focus = paint_text_input_focus(presentation);

    draw_template_nodes(
        frame,
        &presentation.workbench_window_nodes,
        &zero_origin(),
        clip,
        Some(&text_input_focus),
    );
}

fn componentized_chrome_clips(
    layout: &HostWindowLayoutData,
    frame_bounds: &FrameRect,
) -> Option<(FrameRect, FrameRect)> {
    if !visible_rect(&layout.center_band_frame) || !visible_rect(&layout.status_bar_frame) {
        return None;
    }

    let top_height = layout.center_band_frame.y.clamp(0.0, frame_bounds.height);
    Some((
        FrameRect {
            x: frame_bounds.x,
            y: frame_bounds.y,
            width: frame_bounds.width,
            height: top_height,
        },
        layout.status_bar_frame.clone(),
    ))
}

fn visible_rect(rect: &FrameRect) -> bool {
    rect.width > 0.0 && rect.height > 0.0
}

#[cfg(test)]
#[path = "componentized/tests.rs"]
mod tests;
