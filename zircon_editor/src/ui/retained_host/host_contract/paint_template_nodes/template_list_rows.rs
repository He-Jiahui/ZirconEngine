use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{is_component_family, TemplateComponentFamily};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_list_row_style, WorkbenchListRowStyle};
use super::template_list_row_glyphs::push_list_row_adornment;
#[cfg(test)]
#[path = "template_list_rows_tests.rs"]
mod tests;
use super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const LIST_ROW_FONT_SIZE: f32 = 12.0;
const LIST_ROW_TEXT_INSET_X: f32 = 14.0;
const LIST_ROW_TEXT_INSET_Y: f32 = 6.0;
const LIST_ROW_RADIUS: f32 = 4.0;
const LIST_ROW_ADORNMENT_RESERVE: f32 = 26.0;

pub(super) fn push_list_row_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_list_row(node) {
        return false;
    }

    push_list_row_surface(commands, node, rect, clip, order, opacity);
    push_list_row_label(commands, node, rect, clip, order + 2, opacity);
    push_list_row_adornment(
        commands,
        node,
        rect,
        clip,
        order + 3,
        list_row_adornment_color(node),
        opacity,
    );
    true
}

fn is_workbench_list_row(node: &TemplatePaneNodeData) -> bool {
    is_component_family(node, TemplateComponentFamily::ListRow)
        && !node.control_id.as_str().ends_with("Title")
}

fn push_list_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(background) = list_row_background(node) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        list_row_border(node),
        list_row_border_width(node),
        LIST_ROW_RADIUS,
        opacity,
    ));
}

fn push_list_row_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + LIST_ROW_TEXT_INSET_X,
            y: rect.y + LIST_ROW_TEXT_INSET_Y,
            width: (rect.width - LIST_ROW_TEXT_INSET_X - LIST_ROW_ADORNMENT_RESERVE).max(1.0),
            height: (rect.height - LIST_ROW_TEXT_INSET_Y * 2.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        label,
        list_row_text_color(node),
        LIST_ROW_FONT_SIZE,
        LIST_ROW_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn list_row_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    list_row_style(node).background
}

fn list_row_border(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    list_row_style(node).border
}

fn list_row_border_width(node: &TemplatePaneNodeData) -> f32 {
    list_row_style(node).border_width
}

fn list_row_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    list_row_style(node).text
}

fn list_row_adornment_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    list_row_style(node).adornment
}

fn list_row_style(node: &TemplatePaneNodeData) -> WorkbenchListRowStyle {
    select_workbench_list_row_style(node)
}
