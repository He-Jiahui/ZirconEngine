use std::collections::{BTreeMap, BTreeSet};

use super::super::super::super::data::{
    FrameRect, HostWindowLayoutData, HostWindowPresentationData, TemplatePaneNodeData,
};
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_template_nodes::{
    TemplateNodePaintTransform, draw_template_nodes, draw_template_nodes_with_transform,
    has_template_nodes, is_viewport_fallback_scene_node,
};
use super::super::super::root_frames::{resolve_root_frames, zero_origin};
use super::super::{chrome, dock_layer, resize};
use super::modal;
use super::page_overflow::draw_host_page_overflow_menu;
use super::root_template::{draw_root_template_overlay, frame_bounds};

const EXTENSION_MODULE_WORKSPACES_HOST_CONTROL_ID: &str = "WorkbenchExtensionModuleWorkspacesHost";

struct ComponentizedChromeFallbackTransform {
    suppress_viewport_fallback: bool,
}

impl ComponentizedChromeFallbackTransform {
    fn from_presentation(presentation: &HostWindowPresentationData) -> Self {
        Self {
            suppress_viewport_fallback: presentation
                .viewport_image
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
    let Some(workspace_region) =
        componentized_extension_workspace_region(presentation, frame_bounds)
    else {
        return false;
    };
    let subtree = ExtensionWorkspaceSubtree::from_presentation(
        presentation,
        workspace_region.root_node_id.as_str(),
    );

    draw_template_nodes_with_transform(
        frame,
        &presentation.workbench_window_nodes,
        &zero_origin(),
        &workspace_region.clip,
        Some(&presentation.text_input_focus),
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
    clip: FrameRect,
}

struct ExtensionWorkspaceSubtree {
    included_node_ids: BTreeSet<String>,
}

impl ExtensionWorkspaceSubtree {
    fn from_presentation(presentation: &HostWindowPresentationData, root_node_id: &str) -> Self {
        let nodes = (0..presentation.workbench_window_nodes.row_count())
            .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
            .collect::<Vec<_>>();
        let parents_by_node_id = nodes
            .iter()
            .filter_map(|node| {
                (!node.parent_node_id.is_empty()).then(|| {
                    (
                        node.node_id.as_str().to_string(),
                        node.parent_node_id.as_str().to_string(),
                    )
                })
            })
            .collect::<BTreeMap<_, _>>();
        let included_node_ids = nodes
            .iter()
            .filter_map(|node| {
                reaches_subtree_root(node.node_id.as_str(), root_node_id, &parents_by_node_id)
                    .then(|| node.node_id.as_str().to_string())
            })
            .collect();
        Self { included_node_ids }
    }
}

impl TemplateNodePaintTransform for ExtensionWorkspaceSubtree {
    fn transform(
        &self,
        node: TemplatePaneNodeData,
        clip: FrameRect,
    ) -> Option<(TemplatePaneNodeData, FrameRect)> {
        self.included_node_ids
            .contains(node.node_id.as_str())
            .then_some((node, clip))
    }
}

fn reaches_subtree_root(
    node_id: &str,
    root_node_id: &str,
    parents_by_node_id: &BTreeMap<String, String>,
) -> bool {
    let mut current_node_id = node_id;
    let mut visited_node_ids = BTreeSet::new();
    loop {
        if current_node_id == root_node_id {
            return true;
        }
        if !visited_node_ids.insert(current_node_id) {
            return false;
        }
        let Some(parent_node_id) = parents_by_node_id.get(current_node_id) else {
            return false;
        };
        current_node_id = parent_node_id;
    }
}

fn componentized_extension_workspace_region(
    presentation: &HostWindowPresentationData,
    frame_bounds: &FrameRect,
) -> Option<ExtensionWorkspacePaintRegion> {
    let nodes = (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .collect::<Vec<_>>();
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
        clip: intersect_rect(&workspace_frame, frame_bounds)?,
    })
}

fn active_extension_workspace_root_node_id(
    nodes: &[TemplatePaneNodeData],
    module_workspaces_host_node_id: &str,
) -> Option<String> {
    let nodes_by_node_id = nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let parents_by_node_id = nodes
        .iter()
        .filter_map(|node| {
            (!node.parent_node_id.is_empty()).then(|| {
                (
                    node.node_id.as_str().to_string(),
                    node.parent_node_id.as_str().to_string(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();

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
                    &parents_by_node_id,
                )
        })
        .map(|node| node.node_id.as_str().to_string())
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
        draw_template_nodes_with_transform(
            frame,
            &presentation.workbench_window_nodes,
            &zero_origin(),
            frame_bounds,
            Some(&presentation.text_input_focus),
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

    draw_template_nodes(
        frame,
        &presentation.workbench_window_nodes,
        &zero_origin(),
        clip,
        Some(&presentation.text_input_focus),
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
