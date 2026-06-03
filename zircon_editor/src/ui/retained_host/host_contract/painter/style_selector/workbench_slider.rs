use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style::resolved_style_color;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SLIDER_TRACK: [u8; 4] =
    [54, 64, 70, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SLIDER_TRACK_DISABLED:
    [u8; 4] = [38, 45, 50, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SLIDER_TEXT: [u8; 4] =
    [174, 189, 196, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SLIDER_THUMB: [u8; 4] =
    [201, 242, 246, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SLIDER_HALO: [u8; 4] =
    [53, 199, 208, 58];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SLIDER_TICK: [u8; 4] =
    [80, 96, 106, 255];

const SLIDER_VALUE_SURFACE: [u8; 4] = [17, 22, 26, 255];
const SLIDER_VALUE_BORDER: [u8; 4] = [45, 57, 64, 255];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchSliderStyle {
    pub track: [u8; 4],
    pub fill: [u8; 4],
    pub thumb: [u8; 4],
    pub thumb_outline: [u8; 4],
    pub thumb_halo: Option<[u8; 4]>,
    pub value_surface: [u8; 4],
    pub value_border: [u8; 4],
    pub range_value_border: [u8; 4],
    pub label_text: [u8; 4],
    pub value_text: [u8; 4],
    pub tick: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_slider_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchSliderStyle {
    let state = painter_state_for_node(node).slider_resolved_state();
    let disabled = is_disabled(state);
    let fill = slider_fill_color(node, disabled);

    WorkbenchSliderStyle {
        track: slider_track_color(node, disabled),
        fill,
        thumb: slider_thumb_color(node, disabled),
        thumb_outline: slider_thumb_outline_color(node, fill),
        thumb_halo: slider_thumb_halo_color(node, state),
        value_surface: slider_value_surface(disabled),
        value_border: slider_value_border(state, fill),
        range_value_border: SLIDER_VALUE_BORDER,
        label_text: slider_label_color(node, disabled),
        value_text: slider_value_text(disabled),
        tick: WORKBENCH_SLIDER_TICK,
        state,
    }
}

pub(in crate::ui::retained_host::host_contract::painter) fn is_workbench_slider_state_hot(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}

fn is_disabled(state: UiPainterResolvedState) -> bool {
    state == UiPainterResolvedState::Disabled
}

fn slider_track_color(node: &TemplatePaneNodeData, disabled: bool) -> [u8; 4] {
    if disabled {
        WORKBENCH_SLIDER_TRACK_DISABLED
    } else {
        resolved_style_color(node.button_style.element.background_color.as_ref())
            .unwrap_or(WORKBENCH_SLIDER_TRACK)
    }
}

fn slider_fill_color(node: &TemplatePaneNodeData, disabled: bool) -> [u8; 4] {
    if disabled {
        PALETTE.text_disabled
    } else if matches!(node.validation_level.as_str(), "warning") {
        PALETTE.warning
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else {
        PALETTE.accent
    }
}

fn slider_thumb_color(node: &TemplatePaneNodeData, disabled: bool) -> [u8; 4] {
    if disabled {
        PALETTE.text_disabled
    } else {
        declared_color(node.icon_color).unwrap_or(WORKBENCH_SLIDER_THUMB)
    }
}

fn slider_thumb_outline_color(node: &TemplatePaneNodeData, fill: [u8; 4]) -> [u8; 4] {
    resolved_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(fill)
}

fn slider_thumb_halo_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    declared_color(node.state_layer_color)
        .or_else(|| is_workbench_slider_state_hot(state).then_some(WORKBENCH_SLIDER_HALO))
}

fn slider_value_surface(disabled: bool) -> [u8; 4] {
    if disabled {
        PALETTE.surface_disabled
    } else {
        SLIDER_VALUE_SURFACE
    }
}

fn slider_value_border(state: UiPainterResolvedState, fill: [u8; 4]) -> [u8; 4] {
    if matches!(
        state,
        UiPainterResolvedState::Focused | UiPainterResolvedState::Pressed
    ) {
        fill
    } else {
        SLIDER_VALUE_BORDER
    }
}

fn slider_label_color(node: &TemplatePaneNodeData, disabled: bool) -> [u8; 4] {
    if disabled {
        PALETTE.text_disabled
    } else {
        declared_color(node.label_color).unwrap_or(WORKBENCH_SLIDER_TEXT)
    }
}

fn slider_value_text(disabled: bool) -> [u8; 4] {
    if disabled {
        PALETTE.text_disabled
    } else {
        WORKBENCH_SLIDER_TEXT
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
