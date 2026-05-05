use slint::{Model, ModelRc, SharedString};
use zircon_runtime::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiStateFlags, UiTreeId},
    layout::{UiFrame, UiSize},
    surface::UiSurfaceFrame,
    tree::{UiInputPolicy, UiTemplateNodeMetadata, UiTreeNode},
};

use super::super::data::{FrameRect, PaneData, TemplatePaneNodeData};
use super::surface_frame::hit_test_host_surface_frame;

pub(crate) struct TemplateNodePointerHit {
    pub(crate) control_id: SharedString,
    pub(crate) action_id: SharedString,
    pub(crate) binding_id: SharedString,
    pub(crate) dispatch_kind: SharedString,
}

pub(crate) fn hit_test_pane_template_node(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let nodes = pane_template_nodes(pane)?;
    let surface_frame = pane.body_surface_frame.as_ref()?;
    hit_test_template_nodes(nodes, surface_frame, body, x, y)
}

pub(crate) fn build_pane_template_surface_frame(
    pane: &PaneData,
    surface_size: UiSize,
) -> Option<UiSurfaceFrame> {
    let nodes = pane_template_nodes(pane)?;
    let has_dispatchable = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .any(|node| is_dispatchable(&node));
    has_dispatchable.then(|| template_nodes_surface_frame(nodes, surface_size))
}

fn pane_template_nodes(pane: &PaneData) -> Option<&ModelRc<TemplatePaneNodeData>> {
    match pane.kind.as_str() {
        "Hierarchy" => Some(&pane.hierarchy.nodes),
        "Inspector" => Some(&pane.inspector.nodes),
        "Console" => Some(&pane.console.nodes),
        "Assets" => Some(&pane.assets_activity.nodes),
        "AssetBrowser" => Some(&pane.asset_browser.nodes),
        "Project" | "UiComponentShowcase" => Some(&pane.project_overview.nodes),
        "ModulePlugins" | "RuntimeDiagnostics" => Some(&pane.module_plugins.nodes),
        "BuildExport" => Some(&pane.build_export.nodes),
        "UiAssetEditor" => Some(&pane.ui_asset.nodes),
        "AnimationSequenceEditor" | "AnimationGraphEditor" => Some(&pane.animation.nodes),
        _ => None,
    }
}

fn hit_test_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_frame: &UiSurfaceFrame,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let hit = hit_test_host_surface_frame(surface_frame, origin, x, y)?;
    let row = hit.node_id.0.checked_sub(2)? as usize;
    let node = nodes.row_data(row)?;
    Some(TemplateNodePointerHit {
        control_id: node.control_id,
        action_id: node.action_id,
        binding_id: node.binding_id,
        dispatch_kind: node.dispatch_kind,
    })
}

fn is_dispatchable(node: &TemplatePaneNodeData) -> bool {
    !node.disabled
        && !node.control_id.is_empty()
        && (!node.action_id.is_empty()
            || !node.binding_id.is_empty()
            || !node.dispatch_kind.is_empty())
}

fn template_nodes_surface_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_size: UiSize,
) -> UiSurfaceFrame {
    let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.template_nodes.hit"));
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        surface_size.width.max(1.0),
        surface_size.height.max(1.0),
    );
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("template_nodes/root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    surface.tree.insert_root(root);

    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !is_dispatchable(&node) {
            continue;
        }
        let metadata = UiTemplateNodeMetadata {
            component: node.component_role.to_string(),
            control_id: Some(node.control_id.to_string()),
            ..Default::default()
        };
        let tree_node = UiTreeNode::new(
            UiNodeId::new(row as u64 + 2),
            UiNodePath::new(format!("template_nodes/{}", node.node_id)),
        )
        .with_frame(UiFrame::new(
            node.frame.x,
            node.frame.y,
            node.frame.width,
            node.frame.height,
        ))
        .with_state_flags(UiStateFlags {
            visible: true,
            enabled: !node.disabled,
            clickable: true,
            hoverable: true,
            focusable: true,
            pressed: node.pressed,
            checked: node.checked,
            dirty: false,
        })
        .with_input_policy(UiInputPolicy::Receive)
        .with_template_metadata(metadata);
        let _ = surface.tree.insert_child(UiNodeId::new(1), tree_node);
    }

    surface.rebuild();
    surface.surface_frame()
}
