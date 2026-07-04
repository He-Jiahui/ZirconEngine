use super::super::palette::workbench_dropdown_palette;
use super::super::state::is_unavailable_dropdown_state;
use super::declared::declared_style_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_border(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_dropdown_palette();
    let color = if is_unavailable_dropdown_state(state) {
        palette.disabled_border
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        palette.error_border
    } else {
        match state {
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open => palette.focus_border,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => palette.hover_border,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                palette.disabled_border
            }
            UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => palette.border,
        }
    };
    if is_unavailable_dropdown_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(color)
    }
}
