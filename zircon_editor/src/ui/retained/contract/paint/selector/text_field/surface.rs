//! Authored colors follow the same state cascade as the runtime TextField painter.
//! Unavailable input admission is independent from disabled visual overrides.

use super::colors::{authored_field_color, declared_style_color};
use super::palette::workbench_text_field_palette;
use super::state::is_unavailable_text_field_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

fn field_hot(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> bool {
    node.hovered
        || node.dragging
        || node.drop_hovered
        || node.active_drag_target
        || matches!(
            state,
            UiPainterResolvedState::Hovered
                | UiPainterResolvedState::Dragging
                | UiPainterResolvedState::DropHovered
        )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_surface(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    let (keys, fallback): (&[&str], [u8; 4]) = if is_unavailable_text_field_state(state) {
        (&["disabled_background_color"], palette.disabled_surface)
    } else if state == UiPainterResolvedState::Pressed {
        (&["pressed_background_color"], palette.focused_surface)
    } else if state == UiPainterResolvedState::Focused {
        (
            &[
                "focused_background_color",
                "focus_background_color",
                "background_color",
            ],
            palette.focused_surface,
        )
    } else if field_hot(node, state) {
        (&["hover_background_color"], palette.hover_surface)
    } else {
        (&["background_color"], palette.surface)
    };
    authored_field_color(node, keys)
        .or_else(|| {
            keys.contains(&"background_color")
                .then(|| declared_style_color(node.button_style.element.background_color.as_ref()))
                .flatten()
        })
        .unwrap_or(fallback)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_border(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    let has_error = matches!(node.validation_level.as_str(), "error" | "danger");
    let (keys, fallback): (&[&str], [u8; 4]) = if is_unavailable_text_field_state(state) {
        (&["disabled_border_color"], palette.disabled_border)
    } else if has_error {
        (&["error_border_color"], palette.error)
    } else if matches!(
        state,
        UiPainterResolvedState::Pressed | UiPainterResolvedState::Focused
    ) {
        (&["focus_border_color"], palette.focus_border)
    } else if field_hot(node, state) {
        (&["hover_border_color"], palette.hover_border)
    } else {
        (&["border_color"], palette.border)
    };
    authored_field_color(node, keys)
        .or_else(|| {
            (keys == ["border_color"])
                .then(|| declared_style_color(node.button_style.element.border_color.as_ref()))
                .flatten()
        })
        .unwrap_or(fallback)
}
