use super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_node_labels::{template_node_has_label, template_node_label};
use super::template_style::text_color;

mod command;
mod eligibility;
mod geometry;
mod metrics;

use command::{is_paintable_text_slot, push_text_command};
use eligibility::{should_skip_template_text, should_skip_template_text_before_label};
use geometry::text_rect_for_node;
use metrics::node_font_size;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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
    if should_skip_template_text_before_label(
        node,
        property_row_text_painted,
        table_row_text_painted,
    ) {
        return;
    }

    if !template_node_has_label(node, text_input_focus) {
        return;
    }

    let text_rect = text_rect_for_node(node, rect);
    let font_size = node_font_size(node, text_rect.height);
    if !is_paintable_text_slot(&text_rect, clip, font_size) {
        return;
    }

    let label = template_node_label(node, text_input_focus);
    if should_skip_template_text(
        node,
        &label,
        property_row_text_painted,
        table_row_text_painted,
    ) {
        return;
    }

    push_text_command(
        commands,
        &text_rect,
        clip,
        order,
        label,
        text_color(node),
        font_size,
        node_text_paint_style(node),
        opacity,
    );
}

fn node_text_paint_style(node: &TemplatePaneNodeData) -> UiTextRunPaintStyle {
    UiTextRunPaintStyle {
        code: node
            .component_variant
            .split_whitespace()
            .any(|variant| variant.eq_ignore_ascii_case("code")),
        ..UiTextRunPaintStyle::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_component_variant_selects_runtime_code_text_style() {
        let code = TemplatePaneNodeData {
            component_variant: "outlined CODE compact".into(),
            ..TemplatePaneNodeData::default()
        };

        assert!(node_text_paint_style(&code).code);
        assert!(!node_text_paint_style(&TemplatePaneNodeData::default()).code);
    }
}
