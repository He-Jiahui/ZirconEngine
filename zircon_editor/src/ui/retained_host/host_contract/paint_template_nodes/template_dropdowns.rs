use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_dropdown_style, WorkbenchDropdownStyle};
#[cfg(test)]
#[path = "template_dropdowns_tests.rs"]
mod tests;
use super::template_dropdown_glyphs::{push_dropdown_chevron, DROPDOWN_CHEVRON_RESERVE};
use super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DROPDOWN_FONT_SIZE: f32 = 11.0;
const DROPDOWN_LINE_HEIGHT: f32 = DROPDOWN_FONT_SIZE * 1.25;
const DROPDOWN_RADIUS: f32 = 4.0;
const DROPDOWN_TEXT_LEFT: f32 = 10.0;

pub(super) fn push_dropdown_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_dropdown(node) {
        return false;
    }
    let rect = dropdown_paint_rect(node, rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }
    let style = dropdown_style(node);

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        1.0,
        DROPDOWN_RADIUS,
        opacity,
    ));
    push_dropdown_label(commands, node, &rect, clip, order + 2, opacity, &style);
    push_dropdown_chevron(commands, &rect, clip, order + 3, opacity, &style);
    true
}

pub(super) fn dropdown_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    rect
}

fn is_workbench_dropdown(node: &TemplatePaneNodeData) -> bool {
    uses_workbench_visual_language(node)
        && is_component_family(node, TemplateComponentFamily::Dropdown)
}

fn push_dropdown_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
) {
    let label = dropdown_label(node);
    if label.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + DROPDOWN_TEXT_LEFT,
            y: rect.y + (rect.height - DROPDOWN_LINE_HEIGHT).max(0.0) * 0.5,
            width: (rect.width - DROPDOWN_TEXT_LEFT - DROPDOWN_CHEVRON_RESERVE).max(1.0),
            height: DROPDOWN_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        style.text,
        DROPDOWN_FONT_SIZE,
        DROPDOWN_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn dropdown_label(node: &TemplatePaneNodeData) -> String {
    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        return label;
    }
    node.options
        .row_data(0)
        .map(|value| value.to_string())
        .unwrap_or_default()
}

fn dropdown_label_is_placeholder(node: &TemplatePaneNodeData) -> bool {
    template_node_label(node, None).trim().is_empty()
}

fn dropdown_style(node: &TemplatePaneNodeData) -> WorkbenchDropdownStyle {
    select_workbench_dropdown_style(node, dropdown_label_is_placeholder(node))
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.max(1.0),
    }
}
