use crate::ui::retained_host::primitives::ModelRc;
use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{UiFrame, UiSize},
    surface::UiSurfaceFrame,
    tree::{UiInputPolicy, UiTreeNode},
};

use super::super::super::super::data::TemplatePaneNodeData;
use super::dispatch::is_dispatchable;
use super::node::template_surface_tree_node;

pub(in crate::ui::retained_host::host_contract) fn build_template_surface_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_size: UiSize,
) -> Option<UiSurfaceFrame> {
    let has_dispatchable = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .any(|node| is_dispatchable(&node));
    has_dispatchable.then(|| template_nodes_surface_frame(nodes, surface_size))
}

pub(in crate::ui::retained_host::host_contract) fn template_nodes_surface_frame(
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
    surface.tree.insert_root(template_surface_root(root_frame));

    for row in 0..nodes.row_count() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !is_dispatchable(&node) {
            continue;
        }
        let _ = surface
            .tree
            .insert_child(UiNodeId::new(1), template_surface_tree_node(row, node));
    }

    surface.rebuild();
    surface.surface_frame()
}

fn template_surface_root(root_frame: UiFrame) -> UiTreeNode {
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("template_nodes/root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    root
}
