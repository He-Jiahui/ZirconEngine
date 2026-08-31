use std::sync::Arc;

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

#[cfg(test)]
thread_local! {
    static TEMPLATE_SURFACE_FRAME_BUILD_COUNT: std::cell::Cell<u64> = const {
        std::cell::Cell::new(0)
    };
}

pub(in crate::ui::retained_host::host_contract) fn build_template_surface_frame(
    nodes: &ModelRc<TemplatePaneNodeData>,
    surface_size: UiSize,
) -> Option<Arc<UiSurfaceFrame>> {
    let mut dispatchable_nodes = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| is_dispatchable(node));
    let first_dispatchable = dispatchable_nodes.next()?;

    #[cfg(test)]
    TEMPLATE_SURFACE_FRAME_BUILD_COUNT.with(|count| count.set(count.get().saturating_add(1)));

    let mut surface = UiSurface::new(UiTreeId::new("zircon.editor.template_nodes.hit"));
    let root_frame = UiFrame::new(
        0.0,
        0.0,
        surface_size.width.max(1.0),
        surface_size.height.max(1.0),
    );
    surface.tree.insert_root(template_surface_root(root_frame));

    for (row, node) in std::iter::once(first_dispatchable).chain(dispatchable_nodes) {
        let _ = surface
            .tree
            .insert_child(UiNodeId::new(1), template_surface_tree_node(row, node));
    }

    surface.rebuild();
    Some(surface.surface_frame())
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn reset_template_surface_frame_build_count() {
    TEMPLATE_SURFACE_FRAME_BUILD_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn template_surface_frame_build_count() -> u64 {
    TEMPLATE_SURFACE_FRAME_BUILD_COUNT.with(std::cell::Cell::get)
}

fn template_surface_root(root_frame: UiFrame) -> UiTreeNode {
    let mut root = UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("template_nodes/root"))
        .with_frame(root_frame)
        .with_clip_to_bounds(true)
        .with_input_policy(UiInputPolicy::Ignore);
    root.layout_cache.clip_frame = Some(root_frame);
    root
}
