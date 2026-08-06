mod hit;
mod index;
mod model;
mod pane_nodes;
mod popup_rows;
mod surface_frame_builder;

use zircon_runtime_interface::ui::{layout::UiSize, surface::UiSurfaceFrame};

use super::super::data::{FrameRect, HostWindowPresentationData, PaneData};
use super::super::template_geometry::template_nodes_bounds;
use hit::{
    hit_test_template_nodes, hit_test_workbench_template_nodes,
    hit_test_workbench_template_nodes_with_index,
};
pub(crate) use index::HostWorkbenchHitIndex;
pub(crate) use model::TemplateNodePointerHit;
use pane_nodes::pane_template_nodes;
use surface_frame_builder::build_template_surface_frame;

pub(crate) fn hit_test_pane_template_node(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let nodes = pane_template_nodes(pane)?;
    let surface_frame = pane.body_surface_frame.as_ref()?;
    let mut hit = hit_test_template_nodes(nodes, surface_frame, body, x, y)?;
    hit.pane_id = pane.id.clone();
    Some(hit)
}

pub(crate) fn hit_test_workbench_window_template_node(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    let nodes = &presentation.workbench_window_nodes;
    let bounds = template_nodes_bounds(nodes)?;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: bounds.width.max(bounds.x + bounds.width).max(1.0),
        height: bounds.height.max(bounds.y + bounds.height).max(1.0),
    };
    hit_test_workbench_template_nodes(nodes, &origin, x, y)
}

pub(crate) fn hit_test_workbench_window_template_node_with_index(
    presentation: &HostWindowPresentationData,
    index: &HostWorkbenchHitIndex,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    hit_test_workbench_template_nodes_with_index(&presentation.workbench_window_nodes, index, x, y)
}

pub(crate) fn build_pane_template_surface_frame(
    pane: &PaneData,
    surface_size: UiSize,
) -> Option<UiSurfaceFrame> {
    build_template_surface_frame(pane_template_nodes(pane)?, surface_size)
}

#[cfg(test)]
#[path = "template_node_tests.rs"]
mod tests;
