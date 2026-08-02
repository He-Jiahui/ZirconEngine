mod geometry;
mod identity;
mod metrics;
mod palette;
mod points;
mod surface;
mod text;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use geometry::{SampleGridGeometry, has_paintable_sample_grid_extent};
use identity::is_sample_grid;
use points::push_sample_points;
use surface::push_sample_grid_surface;
use text::push_sample_grid_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_sample_grid_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_sample_grid(node) {
        return false;
    }
    if !has_paintable_sample_grid_extent(rect) {
        return true;
    }

    let geometry = SampleGridGeometry::from_frame(rect);
    if geometry.plot.width <= 0.0 || geometry.plot.height <= 0.0 {
        return true;
    }
    push_sample_grid_surface(commands, node, &geometry, clip, order, opacity);
    push_sample_grid_text(commands, node, &geometry, clip, order, opacity);
    push_sample_points(commands, node, &geometry, clip, order, opacity);
    true
}

#[cfg(test)]
#[path = "template_sample_grid_tests/mod.rs"]
mod tests;
