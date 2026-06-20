use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::identity::is_table_header;
use super::super::style::table_row_style;
use super::geometry::table_action_rect;
use super::glyphs::{push_table_gear, push_table_kebab};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_action(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let action_rect = table_action_rect(node, rect);
    let action_color = table_row_style(node).action;
    if is_table_header(node) {
        push_table_gear(commands, &action_rect, clip, order, action_color, opacity);
    } else {
        push_table_kebab(commands, &action_rect, clip, order, action_color, opacity);
    }
}
