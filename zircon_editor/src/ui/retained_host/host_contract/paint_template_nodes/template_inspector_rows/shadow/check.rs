use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_geometry::shadow_check_rect;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_geometry::is_paintable_rect;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_glyphs::push_inspector_check_tick;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_kind::bool_value;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;

use super::super::primitives::push_nested_label;
use super::super::style::{inspector_row_palette, resource_glyph_color, resource_label_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shadow_check_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let palette = inspector_row_palette();
    let metrics = current_host_metrics();
    push_nested_label(
        commands,
        rect,
        clip,
        order,
        node.text.trim(),
        resource_label_color(node),
        opacity,
    );
    let check = shadow_check_rect(node, rect);
    if !is_paintable_rect(&check) {
        return;
    }
    let checked = bool_value(node.value_text.trim()) || node.checked || node.selected;
    commands.push(HostPaintCommand::quad(
        check.clone(),
        Some(clip.clone()),
        order + 1,
        Some(if checked {
            palette.checked_surface
        } else {
            palette.field_surface
        }),
        Some(if checked {
            palette.checked_border
        } else {
            palette.field_border
        }),
        metrics.border_width,
        metrics.radius_control,
        opacity,
    ));
    if checked {
        push_inspector_check_tick(
            commands,
            &check,
            clip,
            order + 2,
            resource_glyph_color(node),
            opacity,
        );
    }
}
