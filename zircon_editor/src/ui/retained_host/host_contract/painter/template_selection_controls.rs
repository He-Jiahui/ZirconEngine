use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{template_component_family, TemplateComponentFamily};
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    is_workbench_selection_state_hot, painter_state_for_node,
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
use super::theme::PALETTE;
use zircon_runtime_interface::ui::{style::UiPainterResolvedState, surface::UiTextRunPaintStyle};

const SELECTION_MARK_INSET_X: f32 = 10.0;
const SELECTION_MARK_SIZE: f32 = 16.0;
const SELECTION_LABEL_GAP: f32 = 9.0;
const SELECTION_TEXT_INSET_Y: f32 = 5.0;
const SELECTION_FONT_SIZE: f32 = 11.0;
const RADIO_DOT_SIZE: f32 = 7.0;
const TOGGLE_TRACK_WIDTH: f32 = 34.0;
const TOGGLE_TRACK_HEIGHT: f32 = 18.0;
const TOGGLE_THUMB_SIZE: f32 = 14.0;
const TOGGLE_RIGHT_INSET: f32 = 8.0;
const TOGGLE_THUMB_INSET: f32 = 2.0;

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

fn leading_mark_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let mark_size = selection_mark_size(node);
    FrameRect {
        x: rect.x + SELECTION_MARK_INSET_X,
        y: rect.y + (rect.height - mark_size).max(0.0) * 0.5,
        width: mark_size,
        height: mark_size,
    }
}

fn label_rect_after_mark(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    mark: &FrameRect,
) -> FrameRect {
    let x = mark.x + mark.width + selection_label_gap(node);
    FrameRect {
        x,
        y: rect.y + SELECTION_TEXT_INSET_Y,
        width: (rect.x + rect.width - x - SELECTION_MARK_INSET_X).max(1.0),
        height: (rect.height - SELECTION_TEXT_INSET_Y * 2.0).max(1.0),
    }
}

fn selection_mark_size(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        SELECTION_MARK_SIZE
    }
}

fn selection_label_gap(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_x > 0.0 {
        node.layout_content_offset_x
    } else {
        SELECTION_LABEL_GAP
    }
}

fn toggle_track_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let track_width =
        toggle_track_width(node).min((rect.width - SELECTION_MARK_INSET_X * 2.0).max(1.0));
    let track_height = toggle_track_height(node).min(rect.height.max(1.0));
    FrameRect {
        x: (rect.x + rect.width - TOGGLE_RIGHT_INSET - track_width).max(rect.x),
        y: rect.y + (rect.height - track_height).max(0.0) * 0.5,
        width: track_width,
        height: track_height,
    }
}

fn toggle_thumb_rect(node: &TemplatePaneNodeData, track: &FrameRect) -> FrameRect {
    let thumb_size = toggle_thumb_size(node)
        .min(track.width)
        .min(track.height)
        .max(1.0);
    let available = (track.width - thumb_size - TOGGLE_THUMB_INSET * 2.0).max(0.0);
    let offset = if node.checked || node.selected {
        available
    } else {
        0.0
    };
    FrameRect {
        x: track.x + TOGGLE_THUMB_INSET + offset,
        y: track.y + (track.height - thumb_size).max(0.0) * 0.5,
        width: thumb_size,
        height: thumb_size,
    }
}

fn toggle_track_width(node: &TemplatePaneNodeData) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        TOGGLE_TRACK_WIDTH
    }
}

fn toggle_track_height(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_content_offset_y > 0.0 {
        node.layout_content_offset_y
    } else {
        TOGGLE_TRACK_HEIGHT
    }
}

fn toggle_thumb_size(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        TOGGLE_THUMB_SIZE
    }
}

fn radio_dot_size(node: &TemplatePaneNodeData) -> f32 {
    if node.value_number > 0.0 {
        node.value_number
    } else {
        RADIO_DOT_SIZE
    }
}

fn centered_square(rect: &FrameRect, size: f32) -> FrameRect {
    let size = size.min(rect.width).min(rect.height).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
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

fn selection_visual_disabled(node: &TemplatePaneNodeData) -> bool {
    matches!(
        selection_visual_state(node),
        UiPainterResolvedState::Disabled
    )
}

fn selection_visual_hot(node: &TemplatePaneNodeData) -> bool {
    is_workbench_selection_state_hot(selection_visual_state(node))
}

fn selection_style(
    node: &TemplatePaneNodeData,
    kind: SelectionStyleKind,
) -> WorkbenchSelectionControlStyle {
    select_workbench_selection_control_style(node, kind)
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use zircon_runtime_interface::ui::style::{
        ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
    };

    #[test]
    fn selection_control_kind_matches_roles_and_workbench_ids() {
        assert_eq!(
            selection_control_kind(&node_with_role("Checkbox", "checkbox", "Custom")),
            Some(SelectionControlKind::Checkbox)
        );
        assert_eq!(
            selection_control_kind(&node_with_role("Radio", "radio", "Custom")),
            Some(SelectionControlKind::Radio)
        );
        assert_eq!(
            selection_control_kind(&node_with_role("Toggle", "toggle", "Custom")),
            Some(SelectionControlKind::Toggle)
        );
        assert_eq!(
            selection_control_kind(&node_with_role("Mount", "", "WorkbenchToggleOn")),
            Some(SelectionControlKind::Toggle)
        );
    }

    #[test]
    fn toggle_thumb_moves_to_checked_end_of_right_aligned_track() {
        let rect = FrameRect {
            x: 4.0,
            y: 6.0,
            width: 96.0,
            height: 28.0,
        };
        let node = TemplatePaneNodeData::default();
        let track = toggle_track_rect(&node, &rect);
        let unchecked = toggle_thumb_rect(&node, &track);
        let checked = toggle_thumb_rect(
            &TemplatePaneNodeData {
                checked: true,
                ..TemplatePaneNodeData::default()
            },
            &track,
        );

        assert_eq!(track.x, 58.0);
        assert_eq!(track.width, TOGGLE_TRACK_WIDTH);
        assert!(checked.x > unchecked.x);
        assert_eq!(unchecked.y, checked.y);
    }

    #[test]
    fn toggle_honors_declared_track_and_thumb_metrics() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 96.0,
            height: 28.0,
        };
        let node = TemplatePaneNodeData {
            value_number: 40.0,
            layout_icon_size: 12.0,
            layout_content_offset_x: 8.0,
            layout_content_offset_y: 16.0,
            ..node_with_role("Toggle", "toggle", "WorkbenchToggleCustom")
        };
        let checked = TemplatePaneNodeData {
            checked: true,
            ..node.clone()
        };
        let track = toggle_track_rect(&node, &rect);
        let unchecked_thumb = toggle_thumb_rect(&node, &track);
        let checked_thumb = toggle_thumb_rect(&checked, &track);

        assert_eq!(track.x, 48.0);
        assert_eq!(track.y, 6.0);
        assert_eq!(track.width, 40.0);
        assert_eq!(track.height, 16.0);
        assert_eq!(unchecked_thumb.x, 50.0);
        assert_eq!(unchecked_thumb.y, 8.0);
        assert_eq!(unchecked_thumb.width, 12.0);
        assert_eq!(checked_thumb.x, 74.0);
        assert_eq!(selection_label_gap(&node), 8.0);
    }

    #[test]
    fn toggle_consumes_declared_track_border_and_thumb_tones() {
        let checked = TemplatePaneNodeData {
            checked: true,
            selected: true,
            button_style: resolved_background_foreground_and_border(
                [53, 199, 208, 255],
                [255, 255, 255, 255],
                [49, 191, 201, 255],
            ),
            ..node_with_role("Toggle", "toggle", "WorkbenchToggleOn")
        };
        let unchecked = TemplatePaneNodeData {
            button_style: resolved_background_foreground_and_border(
                [15, 20, 23, 255],
                [124, 135, 142, 255],
                [53, 64, 71, 255],
            ),
            ..node_with_role("Toggle", "toggle", "WorkbenchToggleOff")
        };

        assert_eq!(toggle_track_color(&checked), [53, 199, 208, 255]);
        assert_eq!(toggle_thumb_color(&checked), [255, 255, 255, 255]);
        assert_eq!(control_border_color(&checked), [49, 191, 201, 255]);
        assert_eq!(toggle_track_color(&unchecked), [15, 20, 23, 255]);
        assert_eq!(toggle_thumb_color(&unchecked), [124, 135, 142, 255]);
        assert_eq!(control_border_color(&unchecked), [53, 64, 71, 255]);
    }

    #[test]
    fn checkbox_radio_marks_use_showcase_metrics_and_tones() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 140.0,
            height: 28.0,
        };
        let unchecked = node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOff");
        let mark = leading_mark_rect(&unchecked, &rect);
        let label = label_rect_after_mark(&unchecked, &rect, &mark);
        let checked_radio = TemplatePaneNodeData {
            checked: true,
            selected: true,
            ..node_with_role("Radio", "radio", "WorkbenchRadioOn")
        };
        let dot = centered_square(&mark, radio_dot_size(&checked_radio));

        assert_eq!(mark.x, 10.0);
        assert_eq!(mark.y, 6.0);
        assert_eq!(mark.width, 16.0);
        assert_eq!(mark.height, 16.0);
        assert_eq!(label.x, 35.0);
        assert_eq!(checkbox_background(&unchecked), SELECTION_MARK_IDLE_FILL);
        assert_eq!(
            checkbox_border_color(&unchecked),
            SELECTION_MARK_IDLE_BORDER
        );
        assert_eq!(
            selection_mark_label_color(&unchecked),
            SELECTION_LABEL_MUTED
        );
        assert_eq!(radio_background(&checked_radio), RADIO_CHECKED_FILL);
        assert_eq!(radio_border_color(&checked_radio), RADIO_CHECKED_BORDER);
        assert_eq!(dot.width, RADIO_DOT_SIZE);
        assert_eq!(dot.height, RADIO_DOT_SIZE);
    }

    #[test]
    fn checkbox_radio_marks_consume_declared_style_and_label_colors() {
        let unchecked = TemplatePaneNodeData {
            label_color: crate::ui::retained_host::primitives::Color::from_rgb_u8(131, 141, 148),
            button_style: resolved_background_and_border([19, 25, 29, 255], [55, 65, 72, 255]),
            ..node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOff")
        };
        let unchecked_radio = TemplatePaneNodeData {
            button_style: resolved_background_and_border([19, 25, 29, 255], [55, 65, 72, 255]),
            ..node_with_role("Radio", "radio", "WorkbenchRadioOff")
        };
        let checked = TemplatePaneNodeData {
            checked: true,
            selected: true,
            button_style: resolved_background_and_border([33, 160, 169, 255], [34, 161, 170, 255]),
            ..node_with_role("Radio", "radio", "WorkbenchRadioOn")
        };

        assert_eq!(checkbox_background(&unchecked), [19, 25, 29, 255]);
        assert_eq!(checkbox_border_color(&unchecked), [55, 65, 72, 255]);
        assert_eq!(selection_mark_label_color(&unchecked), [131, 141, 148, 255]);
        assert_eq!(radio_background(&unchecked_radio), [19, 25, 29, 255]);
        assert_eq!(radio_border_color(&unchecked_radio), [55, 65, 72, 255]);
        assert_eq!(radio_background(&checked), [33, 160, 169, 255]);
        assert_eq!(radio_border_color(&checked), [34, 161, 170, 255]);
    }

    #[test]
    fn radio_uses_declared_dot_size_and_color() {
        let node = TemplatePaneNodeData {
            checked: true,
            selected: true,
            value_number: 6.0,
            value_color: crate::ui::retained_host::primitives::Color::from_rgb_u8(67, 216, 226),
            ..node_with_role("Radio", "radio", "WorkbenchRadioOn")
        };

        assert_eq!(radio_dot_size(&node), 6.0);
        assert_eq!(control_accent_color(&node), [67, 216, 226, 255]);
    }

    #[test]
    fn selection_control_uses_shared_selector_for_pressed_checked_border() {
        let node = TemplatePaneNodeData {
            checked: true,
            pressed: true,
            ..node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOn")
        };

        assert_eq!(
            selection_visual_state(&node),
            UiPainterResolvedState::Pressed
        );
        assert_eq!(checkbox_background(&node), CHECKBOX_CHECKED_FILL);
        assert_eq!(checkbox_border_color(&node), PALETTE.focus_ring);
    }

    #[test]
    fn selection_control_honors_declared_mark_size_and_label_gap() {
        let rect = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 140.0,
            height: 28.0,
        };
        let node = TemplatePaneNodeData {
            layout_icon_size: 14.0,
            layout_content_offset_x: 8.0,
            ..node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxCustom")
        };
        let mark = leading_mark_rect(&node, &rect);
        let label = label_rect_after_mark(&node, &rect, &mark);

        assert_eq!(mark.x, 10.0);
        assert_eq!(mark.y, 7.0);
        assert_eq!(mark.width, 14.0);
        assert_eq!(mark.height, 14.0);
        assert_eq!(label.x, 32.0);
    }

    #[test]
    fn selection_control_paints_checked_checkbox_without_full_row_surface() {
        let bytes = paint_template_nodes_for_test(96, 32, model_rc(vec![checkbox_node()]));

        assert!(changed_pixel_count(&bytes, 96, 8, 7, 18, 18) > 0);
        assert_eq!(pixel_at(&bytes, 96, 92, 14), [0, 0, 0, 255]);
    }

    #[test]
    fn selection_control_paints_unchecked_mark_surface_without_row_fill() {
        let bytes =
            paint_template_nodes_for_test(96, 32, model_rc(vec![unchecked_checkbox_node()]));

        assert_eq!(pixel_at(&bytes, 96, 18, 14), SELECTION_MARK_IDLE_FILL);
        assert_eq!(pixel_at(&bytes, 96, 92, 14), [0, 0, 0, 255]);
    }

    fn node_with_role(role: &str, component_role: &str, control_id: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: role.into(),
            component_role: component_role.into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 28.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn checkbox_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "WorkbenchCheckboxOn".into(),
            role: "Checkbox".into(),
            component_role: "checkbox".into(),
            text: "Checkbox".into(),
            checked: true,
            selected: true,
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 96.0,
                height: 28.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn unchecked_checkbox_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "WorkbenchCheckboxOff".into(),
            role: "Checkbox".into(),
            component_role: "checkbox".into(),
            text: "Checkbox".into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 96.0,
                height: 28.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn changed_pixel_count(
        bytes: &[u8],
        frame_width: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> usize {
        let mut changed = 0;
        for py in y..(y + height) {
            for px in x..(x + width) {
                let index = ((py as usize * frame_width as usize) + px as usize) * 4;
                if bytes[index..index + 4] != [0, 0, 0, 255] {
                    changed += 1;
                }
            }
        }
        changed
    }

    fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y as usize * frame_width as usize) + x as usize) * 4;
        [
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]
    }

    fn resolved_background_and_border(background: [u8; 4], border: [u8; 4]) -> ResolvedButtonStyle {
        resolved_background_foreground_and_border(background, [0, 0, 0, 0], border)
    }

    fn resolved_background_foreground_and_border(
        background: [u8; 4],
        foreground: [u8; 4],
        border: [u8; 4],
    ) -> ResolvedButtonStyle {
        ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    background[0],
                    background[1],
                    background[2],
                    background[3],
                ))),
                border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    border[0], border[1], border[2], border[3],
                ))),
                foreground_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    foreground[0],
                    foreground[1],
                    foreground[2],
                    foreground[3],
                ))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
        }
    }
}
