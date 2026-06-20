use super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_node_labels::template_node_label;
use super::template_style::text_color;

mod command;
mod eligibility;
mod geometry;
mod metrics;

use command::push_text_command;
use eligibility::should_skip_template_text;
use geometry::text_rect_for_node;
use metrics::node_font_size;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_text_fallback_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text_input_focus: Option<&HostTextInputFocusData>,
    property_row_text_painted: bool,
    table_row_text_painted: bool,
    opacity: f32,
) {
    let label = template_node_label(node, text_input_focus);
    if should_skip_template_text(
        node,
        &label,
        property_row_text_painted,
        table_row_text_painted,
    ) {
        return;
    }

    let text_rect = text_rect_for_node(node, rect);
    let font_size = node_font_size(node, text_rect.height);
    push_text_command(
        commands,
        &text_rect,
        clip,
        order,
        label,
        text_color(node),
        font_size,
        opacity,
    );
}
