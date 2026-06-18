use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::super::template_component_family::{is_component_family, TemplateComponentFamily};
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    select_workbench_segmented_control_style, WorkbenchSegmentedControlKind as SegmentedStyleKind,
    WorkbenchSegmentedControlStyle, WORKBENCH_SEGMENT_IDLE_BACKGROUND,
};
use super::template_node_labels::template_node_label;
use super::template_segmented_control_geometry::{
    segment_divider_rect, segment_group_label_line_height, segment_label_rect, segment_line_height,
    segment_rect, segmented_body_rect, segmented_group_label_rect, selected_segment_rect,
    selected_segment_underline_rect, tab_label_rect, tab_line_height, tab_paint_rect,
    tab_underline_rect, SEGMENT_FONT_SIZE, SEGMENT_GROUP_LABEL_FONT_SIZE, SEGMENT_RADIUS,
    TAB_FONT_SIZE,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SEGMENT_IDLE_BACKGROUND: [u8; 4] = WORKBENCH_SEGMENT_IDLE_BACKGROUND;

pub(super) fn push_segmented_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_segmented_control(node) {
        let options = segmented_options(node);
        if options.is_empty() {
            return false;
        }
        push_segmented_control(commands, node, rect, clip, order, opacity, &options);
        return true;
    }

    if is_workbench_tab(node) {
        push_workbench_tab(commands, node, rect, clip, order, opacity);
        return true;
    }

    false
}

fn is_segmented_control(node: &TemplatePaneNodeData) -> bool {
    is_component_family(node, TemplateComponentFamily::SegmentedControl)
}

fn is_workbench_tab(node: &TemplatePaneNodeData) -> bool {
    is_component_family(node, TemplateComponentFamily::Tab)
}

fn push_segmented_control(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    options: &[String],
) {
    push_segmented_group_label(commands, node, rect, clip, order + 3, opacity);
    let body_rect = segmented_body_rect(node, rect);
    let style = segmented_control_style(node);
    commands.push(HostPaintCommand::quad(
        body_rect.clone(),
        Some(clip.clone()),
        order,
        style.background,
        style.border,
        style.border_width,
        SEGMENT_RADIUS,
        opacity,
    ));

    let selected = selected_segment_value(node);
    for (index, option) in options.iter().enumerate() {
        let segment = segment_rect(&body_rect, index, options.len());
        if index > 0 {
            push_segment_divider(commands, &segment, clip, order + 1, opacity);
        }
        if option_is_selected(option, selected.as_deref()) {
            push_selected_segment(commands, node, &segment, clip, order + 2, opacity);
        }
        push_segment_label(
            commands,
            node,
            option,
            &segment,
            clip,
            order + 4,
            option_is_selected(option, selected.as_deref()),
            opacity,
        );
    }
}

fn push_segmented_group_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = node.label_text.trim();
    if label.is_empty() {
        return;
    }

    commands.push(HostPaintCommand::text(
        segmented_group_label_rect(rect),
        Some(clip.clone()),
        order,
        label.to_string(),
        segmented_group_label_color(node),
        SEGMENT_GROUP_LABEL_FONT_SIZE,
        segment_group_label_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_workbench_tab(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let rect = tab_paint_rect(node, rect);
    let style = tab_style(node);
    if let Some(background) = style.background {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(background),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
    if node.checked || node.selected {
        commands.push(HostPaintCommand::quad(
            tab_underline_rect(&rect),
            Some(clip.clone()),
            order + 2,
            Some(style.selected_underline),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }

    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        commands.push(HostPaintCommand::text(
            tab_label_rect(&rect),
            Some(clip.clone()),
            order + 3,
            label,
            tab_text_color(node),
            TAB_FONT_SIZE,
            tab_line_height(),
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}

fn push_segment_divider(
    commands: &mut Vec<HostPaintCommand>,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        segment_divider_rect(segment),
        Some(clip.clone()),
        order,
        Some(PALETTE.border),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_selected_segment(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let selected_rect = selected_segment_rect(segment);
    let style = segmented_control_style(node);
    let border_width = style.selected_border_width;
    commands.push(HostPaintCommand::quad(
        selected_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.selected_surface),
        (border_width > 0.0).then_some(style.selected_border),
        border_width,
        (SEGMENT_RADIUS - 1.0).max(0.0),
        opacity,
    ));

    let underline_height = style.selected_underline_height;
    if underline_height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        selected_segment_underline_rect(&selected_rect, underline_height),
        Some(clip.clone()),
        order + 1,
        Some(style.selected_underline),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_segment_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    option: &str,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    selected: bool,
    opacity: f32,
) {
    let label = segment_label(option);
    commands.push(HostPaintCommand::text(
        segment_label_rect(segment),
        Some(clip.clone()),
        order,
        label,
        segment_text_color(node, selected),
        SEGMENT_FONT_SIZE,
        segment_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn segmented_options(node: &TemplatePaneNodeData) -> Vec<String> {
    (0..node.options.row_count())
        .filter_map(|row| node.options.row_data(row))
        .map(|option| option.to_string())
        .filter(|option| !option.trim().is_empty())
        .collect()
}

fn selected_segment_value(node: &TemplatePaneNodeData) -> Option<String> {
    [
        node.value_text.as_str(),
        node.options_text.as_str(),
        node.text.as_str(),
    ]
    .into_iter()
    .find(|value| !value.trim().is_empty())
    .map(|value| value.trim().to_ascii_lowercase())
}

fn option_is_selected(option: &str, selected: Option<&str>) -> bool {
    selected.is_some_and(|value| option.trim().eq_ignore_ascii_case(value))
}

fn segment_label(option: &str) -> String {
    let trimmed = option.trim();
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) => {
            let mut label = first.to_ascii_uppercase().to_string();
            label.push_str(chars.as_str());
            label
        }
        None => String::new(),
    }
}

fn segmented_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    segmented_control_style(node)
        .background
        .unwrap_or(SEGMENT_IDLE_BACKGROUND)
}

fn segmented_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    segmented_control_style(node)
        .border
        .unwrap_or(PALETTE.border)
}

fn segment_text_color(node: &TemplatePaneNodeData, selected: bool) -> [u8; 4] {
    let style = segmented_control_style(node);
    if selected {
        style.selected_text
    } else {
        style.idle_text
    }
}

fn selected_segment_border_width(node: &TemplatePaneNodeData) -> f32 {
    segmented_control_style(node).selected_border_width
}

fn selected_segment_underline_height(node: &TemplatePaneNodeData) -> f32 {
    segmented_control_style(node).selected_underline_height
}

fn selected_segment_underline_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    segmented_control_style(node).selected_underline
}

fn segmented_group_label_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    segmented_control_style(node).group_label
}

fn tab_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    tab_style(node).background
}

fn tab_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    let style = tab_style(node);
    if node.checked || node.selected {
        style.selected_text
    } else {
        style.idle_text
    }
}

fn segmented_control_style(node: &TemplatePaneNodeData) -> WorkbenchSegmentedControlStyle {
    select_workbench_segmented_control_style(node, SegmentedStyleKind::SegmentedControl)
}

fn tab_style(node: &TemplatePaneNodeData) -> WorkbenchSegmentedControlStyle {
    select_workbench_segmented_control_style(node, SegmentedStyleKind::Tab)
}

#[cfg(test)]
#[path = "template_segmented_controls_tests.rs"]
mod tests;
