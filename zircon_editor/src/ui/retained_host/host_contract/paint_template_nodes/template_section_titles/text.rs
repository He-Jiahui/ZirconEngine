use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::geometry::section_label_rect;
use super::style::{section_text_color, section_title_metrics, WorkbenchSectionTitleMetrics};
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
    let metrics = section_title_metrics();
    let text_rect = section_label_rect(rect, has_icon);
    push_text(
        commands,
        text_rect.clone(),
        clip,
        order,
        &label,
        node,
        &metrics,
        opacity,
    );
    if node.font_weight >= 600 {
        push_text(
            commands,
            FrameRect {
                x: text_rect.x + metrics.strong_offset_x,
                ..text_rect
            },
            clip,
            order + 1,
            &label,
            node,
            &metrics,
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
    metrics: &WorkbenchSectionTitleMetrics,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        label.to_string(),
        section_text_color(node),
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
