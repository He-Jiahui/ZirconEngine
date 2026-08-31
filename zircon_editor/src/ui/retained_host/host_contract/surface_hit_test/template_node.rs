mod hit;
mod index;
mod model;
mod pane_index;
mod pane_nodes;
mod popup_rows;
mod route_hit;
mod surface_frame_builder;

use std::sync::Arc;

use zircon_runtime_interface::ui::{layout::UiSize, surface::UiSurfaceFrame};

use crate::ui::retained_host::console_output::ConsoleOutputPaintMetadata;

use super::super::data::{FrameRect, HostWindowPresentationData, PaneData};
use hit::{
    hit_test_console_static_template_nodes, hit_test_scrolled_console_template_nodes,
    hit_test_template_nodes, hit_test_workbench_template_node_for_pointer_move_with_index,
    hit_test_workbench_template_nodes_with_index,
};
pub(crate) use index::HostWorkbenchHitIndex;
pub(crate) use model::{
    TemplateNodePointerHit, TemplateNodePointerMoveHit, TemplateNodePointerMoveKind,
    TemplateNodePointerRouteHit,
};
pub(crate) use pane_index::HostPaneTemplateHitIndex;
use pane_nodes::pane_template_nodes;
use surface_frame_builder::build_template_surface_frame;

pub(crate) fn hit_test_pane_template_node(
    pane: &PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
    console_scroll_px: f32,
) -> Option<TemplateNodePointerHit> {
    hit_test_pane_template_node_borrowed(pane, body, x, y, console_scroll_px)
        .map(|hit| hit.to_owned_hit())
}

pub(crate) fn hit_test_pane_template_node_borrowed<'a>(
    pane: &'a PaneData,
    body: &FrameRect,
    x: f32,
    y: f32,
    console_scroll_px: f32,
) -> Option<TemplateNodePointerRouteHit<'a>> {
    let nodes = pane_template_nodes(pane)?;
    let popup_rows = pane
        .body_template_hit_index
        .as_deref()
        .filter(|index| index.indexes_nodes(nodes));
    if let Some(metadata) = nodes.metadata::<ConsoleOutputPaintMetadata>() {
        let viewport = metadata.viewport();
        let viewport_frame = FrameRect {
            x: body.x + viewport.x,
            y: body.y + viewport.y,
            width: viewport.width,
            height: viewport.height,
        };
        let hit = if super::super::frame_geometry::contains_point(&viewport_frame, x, y) {
            hit_test_scrolled_console_template_nodes(
                nodes,
                metadata,
                body,
                x,
                y,
                console_scroll_px,
                popup_rows,
            )
        } else {
            hit_test_console_static_template_nodes(nodes, metadata, body, x, y, popup_rows)
        }?;
        return Some(hit.with_pane_id(pane.id.as_str()));
    }
    let surface_frame = pane.body_surface_frame.as_ref()?;
    hit_test_template_nodes(nodes, surface_frame, body, x, y, popup_rows)
        .map(|hit| hit.with_pane_id(pane.id.as_str()))
}

pub(crate) fn hit_test_workbench_window_template_node_with_index(
    presentation: &HostWindowPresentationData,
    index: &HostWorkbenchHitIndex,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerHit> {
    hit_test_workbench_template_nodes_with_index(&presentation.workbench_window_nodes, index, x, y)
}

pub(crate) fn hit_test_workbench_window_template_node_for_pointer_move_with_index<'a>(
    presentation: &'a HostWindowPresentationData,
    index: &HostWorkbenchHitIndex,
    x: f32,
    y: f32,
) -> Option<TemplateNodePointerMoveHit<'a>> {
    hit_test_workbench_template_node_for_pointer_move_with_index(
        &presentation.workbench_window_nodes,
        index,
        x,
        y,
    )
}

pub(crate) fn build_pane_template_surface_frame(
    pane: &PaneData,
    surface_size: UiSize,
) -> Option<Arc<UiSurfaceFrame>> {
    build_template_surface_frame(pane_template_nodes(pane)?, surface_size)
}

pub(crate) fn rebuild_pane_template_hit_artifacts(pane: &mut PaneData, surface_size: UiSize) {
    let Some(nodes) = pane_template_nodes(pane).cloned() else {
        pane.body_surface_frame = None;
        pane.body_template_hit_index = None;
        return;
    };
    pane.body_surface_frame = build_template_surface_frame(&nodes, surface_size);
    pane.body_template_hit_index = Some(Arc::new(HostPaneTemplateHitIndex::new(nodes)));
}

#[cfg(test)]
#[path = "template_node_tests.rs"]
mod tests;
