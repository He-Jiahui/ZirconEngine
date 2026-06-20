use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::geometry::section_label_rect;
use super::style::{section_text_color, SECTION_FONT_SIZE, SECTION_LINE_HEIGHT};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_section_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    has_icon: bool,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let text_rect = section_label_rect(rect, has_icon);
    push_text(
        commands,
        text_rect.clone(),
        clip,
        order,
        &label,
        node,
        opacity,
    );
    if node.font_weight >= 600 {
        push_text(
            commands,
            FrameRect {
                x: text_rect.x + 0.45,
                ..text_rect
            },
            clip,
            order + 1,
            &label,
            node,
            opacity,
        );
    }
}

fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    node: &TemplatePaneNodeData,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        label.to_string(),
        section_text_color(node),
        SECTION_FONT_SIZE,
        SECTION_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
