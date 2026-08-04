use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::geometry::{frame_is_within, section_label_rect};
use super::style::{section_text_color, section_title_metrics, WorkbenchSectionTitleMetrics};
use crate::ui::retained_host::host_contract::paint_geometry::intersect;
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
        rect,
        clip,
        order,
        &label,
        node,
        &metrics,
        opacity,
    );
    if node.font_weight >= 600 {
        let strong_width = (text_rect.width - metrics.strong_offset_x).max(0.0);
        push_text(
            commands,
            FrameRect {
                x: text_rect.x + metrics.strong_offset_x,
                width: strong_width,
                ..text_rect
            },
            rect,
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
    section_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    node: &TemplatePaneNodeData,
    metrics: &WorkbenchSectionTitleMetrics,
    opacity: f32,
) {
    if !frame_is_within(section_rect, &rect) || intersect(&rect, clip).is_none() {
        return;
    }
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
