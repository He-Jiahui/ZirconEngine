use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_chip_glyphs::{chip_has_chevron, push_chip_chevron};
use super::geometry::{has_paintable_chip_extent, pixel_aligned_rect};
use super::identity::is_workbench_chip;
use super::layers::{chevron_order, label_order};
use super::style::chip_glyph_color;
use super::surface::push_chip_surface;
use super::text::push_chip_label;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_chip(node) {
        return false;
    }
    let rect = pixel_aligned_rect(rect);
    if !has_paintable_chip_extent(&rect) {
        return true;
    }
    let Some(clip) = intersect(&rect, clip) else {
        return true;
    };

    push_chip_surface(commands, node, &rect, &clip, order, opacity);
    push_chip_label(commands, node, &rect, &clip, label_order(order), opacity);
    if chip_has_chevron(node) {
        push_chip_chevron(
            commands,
            &rect,
            &clip,
            chevron_order(order),
            chip_glyph_color(node),
            opacity,
        );
    }
    true
}
