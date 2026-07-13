mod geometry;
mod identity;
mod keys;
mod metrics;
mod palette;
mod surface;
mod text;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use geometry::TimelineStripGeometry;
use identity::is_timeline_strip;
use keys::push_timeline_keys_and_playhead;
use metrics::timeline_metrics;
use palette::timeline_palette;
use surface::push_timeline_surface;
use text::push_timeline_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_timeline_strip_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_timeline_strip(node) {
        return false;
    }

    let metrics = timeline_metrics();
    let palette = timeline_palette();
    let geometry = TimelineStripGeometry::from_frame(rect, metrics);
    push_timeline_surface(
        commands, node, &geometry, clip, order, opacity, metrics, palette,
    );
    push_timeline_text(
        commands, node, &geometry, clip, order, opacity, metrics, palette,
    );
    push_timeline_keys_and_playhead(
        commands, node, &geometry, clip, order, opacity, metrics, palette,
    );
    true
}

#[cfg(test)]
#[path = "template_timeline_strip_tests/mod.rs"]
mod tests;
