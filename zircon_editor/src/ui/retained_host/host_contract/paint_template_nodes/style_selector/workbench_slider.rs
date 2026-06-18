use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::template_style_color::resolved_style_color;
use super::resolved_state_for_node;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TRACK: [u8; 4] =
    [54, 64, 70, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TRACK_DISABLED:
    [u8; 4] = [38, 45, 50, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TEXT: [u8; 4] =
    [174, 189, 196, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_THUMB: [u8; 4] =
    [201, 242, 246, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_HALO: [u8; 4] =
    [53, 199, 208, 58];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SLIDER_TICK: [u8; 4] =
    [80, 96, 106, 255];

const SLIDER_VALUE_SURFACE: [u8; 4] = [17, 22, 26, 255];
const SLIDER_VALUE_BORDER: [u8; 4] = [45, 57, 64, 255];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSliderStyle {
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_slider_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchSliderStyle {
    let state = resolved_state_for_node(node).slider_resolved_state();
    let unavailable = is_unavailable_slider_state(state);
    let fill = slider_fill_color(node, unavailable);

    WorkbenchSliderStyle {
        track: slider_track_color(node, unavailable),
        fill,
        thumb: slider_thumb_color(node, unavailable),
        thumb_outline: slider_thumb_outline_color(node, state, fill),
        thumb_halo: slider_thumb_halo_color(node, state),
        value_surface: slider_value_surface(unavailable),
        value_border: slider_value_border(state, fill),
        range_value_border: slider_range_value_border(state),
        label_text: slider_label_color(node, unavailable),
        value_text: slider_value_text(unavailable),
        tick: slider_tick_color(unavailable),
        state,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_workbench_slider_state_hot(
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

fn is_unavailable_slider_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn slider_track_color(node: &TemplatePaneNodeData, unavailable: bool) -> [u8; 4] {
    if unavailable {
        WORKBENCH_SLIDER_TRACK_DISABLED
    } else {
        resolved_style_color(node.button_style.element.background_color.as_ref())
            .unwrap_or(WORKBENCH_SLIDER_TRACK)
    }
}

fn slider_fill_color(node: &TemplatePaneNodeData, unavailable: bool) -> [u8; 4] {
    if unavailable {
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

fn slider_thumb_color(node: &TemplatePaneNodeData, unavailable: bool) -> [u8; 4] {
    if unavailable {
        PALETTE.text_disabled
    } else {
        declared_color(node.icon_color).unwrap_or(WORKBENCH_SLIDER_THUMB)
    }
}

fn slider_thumb_outline_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    fill: [u8; 4],
) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        PALETTE.border_disabled
    } else {
        resolved_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(fill)
    }
}

fn slider_thumb_halo_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    if is_unavailable_slider_state(state) {
        None
    } else {
        declared_color(node.state_layer_color)
            .or_else(|| is_workbench_slider_state_hot(state).then_some(WORKBENCH_SLIDER_HALO))
    }
}

fn slider_value_surface(unavailable: bool) -> [u8; 4] {
    if unavailable {
        PALETTE.surface_disabled
    } else {
        SLIDER_VALUE_SURFACE
    }
}

fn slider_value_border(state: UiPainterResolvedState, fill: [u8; 4]) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        return PALETTE.border_disabled;
    }
    if matches!(
        state,
        UiPainterResolvedState::Focused | UiPainterResolvedState::Pressed
    ) {
        fill
    } else {
        SLIDER_VALUE_BORDER
    }
}

fn slider_range_value_border(state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        PALETTE.border_disabled
    } else {
        SLIDER_VALUE_BORDER
    }
}

fn slider_label_color(node: &TemplatePaneNodeData, unavailable: bool) -> [u8; 4] {
    if unavailable {
        PALETTE.text_disabled
    } else {
        declared_color(node.label_color).unwrap_or(WORKBENCH_SLIDER_TEXT)
    }
}

fn slider_value_text(unavailable: bool) -> [u8; 4] {
    if unavailable {
        PALETTE.text_disabled
    } else {
        WORKBENCH_SLIDER_TEXT
    }
}

fn slider_tick_color(unavailable: bool) -> [u8; 4] {
    if unavailable {
        PALETTE.border_disabled
    } else {
        WORKBENCH_SLIDER_TICK
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn slider_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.pressed = true;
        node.drop_hovered = true;
        node.button_style.loading = true;
        node.validation_level = "warning".into();
        node.value_color = Color::from_rgb_u8(53, 199, 208);
        node.icon_color = Color::from_rgb_u8(201, 242, 246);
        node.label_color = Color::from_rgb_u8(174, 189, 196);
        node.state_layer_color = Color::from_argb_u8(58, 53, 199, 208);
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(54, 64, 70, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(53, 199, 208, 255)));

        let style = select_workbench_slider_style(&node);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.track, WORKBENCH_SLIDER_TRACK_DISABLED);
        assert_eq!(style.fill, PALETTE.text_disabled);
        assert_eq!(style.thumb, PALETTE.text_disabled);
        assert_eq!(style.thumb_outline, PALETTE.border_disabled);
        assert_eq!(style.thumb_halo, None);
        assert_eq!(style.value_surface, PALETTE.surface_disabled);
        assert_eq!(style.value_border, PALETTE.border_disabled);
        assert_eq!(style.range_value_border, PALETTE.border_disabled);
        assert_eq!(style.label_text, PALETTE.text_disabled);
        assert_eq!(style.value_text, PALETTE.text_disabled);
        assert_eq!(style.tick, PALETTE.border_disabled);
    }
}
