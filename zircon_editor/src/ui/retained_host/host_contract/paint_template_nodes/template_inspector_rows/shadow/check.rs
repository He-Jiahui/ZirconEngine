use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_geometry::shadow_check_rect;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_glyphs::push_inspector_check_tick;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_inspector_row_kind::bool_value;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::super::primitives::push_nested_label;
use super::super::style::RESOURCE_FIELD_BORDER;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shadow_check_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_nested_label(commands, rect, clip, order, node.text.trim(), opacity);
    let check = shadow_check_rect(node, rect);
    let checked = bool_value(node.value_text.trim()) || node.checked || node.selected;
    commands.push(HostPaintCommand::quad(
        check.clone(),
        Some(clip.clone()),
        order + 1,
        Some(if checked {
            PALETTE.accent_soft
        } else {
            PALETTE.surface_inset
        }),
        Some(if checked {
            PALETTE.accent
        } else {
            RESOURCE_FIELD_BORDER
        }),
        1.0,
        3.0,
        opacity,
    ));
    if checked {
        push_inspector_check_tick(commands, &check, clip, order + 2, opacity);
    }
}
