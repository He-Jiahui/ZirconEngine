use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::super::template_component_family::{template_component_family, TemplateComponentFamily};
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    select_workbench_selection_control_style, WorkbenchSelectionControlKind as SelectionStyleKind,
    WorkbenchSelectionControlStyle,
};
#[cfg(test)]
use super::style_selector::{
    WORKBENCH_CHECKBOX_CHECKED_FILL as CHECKBOX_CHECKED_FILL,
    WORKBENCH_RADIO_CHECKED_BORDER as RADIO_CHECKED_BORDER,
    WORKBENCH_RADIO_CHECKED_FILL as RADIO_CHECKED_FILL,
    WORKBENCH_SELECTION_LABEL_MUTED as SELECTION_LABEL_MUTED,
    WORKBENCH_SELECTION_MARK_IDLE_BORDER as SELECTION_MARK_IDLE_BORDER,
    WORKBENCH_SELECTION_MARK_IDLE_FILL as SELECTION_MARK_IDLE_FILL,
};
use super::template_node_labels::template_node_label;
use super::template_selection_control_geometry::{
    centered_square, label_rect_after_mark, leading_mark_rect, radio_dot_size, selection_label_gap,
    toggle_thumb_rect, toggle_track_rect, SELECTION_MARK_INSET_X, SELECTION_TEXT_INSET_Y,
};
#[cfg(test)]
use super::template_selection_control_geometry::{RADIO_DOT_SIZE, TOGGLE_TRACK_WIDTH};
use zircon_runtime_interface::ui::{style::UiPainterResolvedState, surface::UiTextRunPaintStyle};

const SELECTION_FONT_SIZE: f32 = 11.0;

pub(super) fn push_selection_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match selection_control_kind(node) {
        Some(SelectionControlKind::Checkbox) => {
            push_checkbox(commands, node, rect, clip, order, opacity);
            true
        }
        Some(SelectionControlKind::Radio) => {
            push_radio(commands, node, rect, clip, order, opacity);
            true
        }
        Some(SelectionControlKind::Toggle) => {
            push_toggle(commands, node, rect, clip, order, opacity);
            true
        }
        None => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionControlKind {
    Checkbox,
    Radio,
    Toggle,
}

fn selection_control_kind(node: &TemplatePaneNodeData) -> Option<SelectionControlKind> {
    match template_component_family(node) {
        Some(TemplateComponentFamily::Checkbox) => Some(SelectionControlKind::Checkbox),
        Some(TemplateComponentFamily::Radio) => Some(SelectionControlKind::Radio),
        Some(TemplateComponentFamily::Toggle) => Some(SelectionControlKind::Toggle),
        _ => None,
    }
}

fn push_checkbox(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let mark = leading_mark_rect(node, rect);
    commands.push(HostPaintCommand::quad(
        mark.clone(),
        Some(clip.clone()),
        order,
        Some(checkbox_background(node)),
        Some(checkbox_border_color(node)),
        1.0,
        3.0,
        opacity,
    ));
    if node.checked || node.selected {
        push_checkbox_tick(commands, &mark, clip, order + 1, opacity);
    }
    push_selection_label(
        commands,
        node,
        label_rect_after_mark(node, rect, &mark),
        clip,
        order + 2,
        selection_mark_label_color(node),
        opacity,
    );
}

fn push_radio(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let mark = leading_mark_rect(node, rect);
    commands.push(HostPaintCommand::quad(
        mark.clone(),
        Some(clip.clone()),
        order,
        Some(radio_background(node)),
        Some(radio_border_color(node)),
        1.0,
        mark.height * 0.5,
        opacity,
    ));
    if node.checked || node.selected {
        let dot_size = radio_dot_size(node);
        let dot = centered_square(&mark, dot_size);
        commands.push(HostPaintCommand::quad(
            dot,
            Some(clip.clone()),
            order + 1,
            Some(control_accent_color(node)),
            None,
            0.0,
            dot_size * 0.5,
            opacity,
        ));
    }
    push_selection_label(
        commands,
        node,
        label_rect_after_mark(node, rect, &mark),
        clip,
        order + 2,
        selection_mark_label_color(node),
        opacity,
    );
}

fn push_toggle(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let track = toggle_track_rect(node, rect);
    let label_rect = FrameRect {
        x: rect.x + SELECTION_MARK_INSET_X,
        y: rect.y + SELECTION_TEXT_INSET_Y,
        width: (track.x - rect.x - SELECTION_MARK_INSET_X - selection_label_gap(node)).max(1.0),
        height: (rect.height - SELECTION_TEXT_INSET_Y * 2.0).max(1.0),
    };
    push_selection_label(
        commands,
        node,
        label_rect,
        clip,
        order + 1,
        selection_text_color(node),
        opacity,
    );

    commands.push(HostPaintCommand::quad(
        track.clone(),
        Some(clip.clone()),
        order,
        Some(toggle_track_color(node)),
        Some(control_border_color(node)),
        1.0,
        track.height * 0.5,
        opacity,
    ));
    let thumb = toggle_thumb_rect(node, &track);
    commands.push(HostPaintCommand::quad(
        thumb.clone(),
        Some(clip.clone()),
        order + 2,
        Some(toggle_thumb_color(node)),
        None,
        0.0,
        thumb.height * 0.5,
        opacity,
    ));
}

fn push_selection_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() || rect.width <= 0.5 || rect.height <= 0.5 {
        return;
    }
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        label,
        color,
        SELECTION_FONT_SIZE,
        SELECTION_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_checkbox_tick(
    commands: &mut Vec<HostPaintCommand>,
    mark: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = PALETTE.shell_background;
    for tick in [
        FrameRect {
            x: mark.x + 3.0,
            y: mark.y + 7.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: mark.x + 5.0,
            y: mark.y + 9.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: mark.x + 8.0,
            y: mark.y + 4.0,
            width: 3.0,
            height: 8.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            tick,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn checkbox_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Checkbox).surface
}

fn radio_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Radio).surface
}

fn toggle_track_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).surface
}

fn toggle_thumb_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).thumb
}

fn control_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).border
}

fn checkbox_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Checkbox).border
}

fn radio_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Radio).border
}

fn control_accent_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Radio).accent
}

fn selection_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Toggle).text
}

fn selection_mark_label_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    selection_style(node, SelectionStyleKind::Checkbox).label
}

fn selection_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    selection_style(node, SelectionStyleKind::Checkbox).state
}

#[cfg(test)]
fn selection_visual_unavailable(node: &TemplatePaneNodeData) -> bool {
    matches!(
        selection_visual_state(node),
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn selection_style(
    node: &TemplatePaneNodeData,
    kind: SelectionStyleKind,
) -> WorkbenchSelectionControlStyle {
    select_workbench_selection_control_style(node, kind)
}

#[cfg(test)]
#[path = "template_selection_controls_tests.rs"]
mod tests;
