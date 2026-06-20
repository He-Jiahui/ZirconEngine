use super::super::model::WorkbenchIconButtonContext;
use super::super::palette::{ICON_MUTED, ICON_NORMAL};
use super::super::state::is_unavailable_icon_button_state;
use super::declared::declared_icon_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_glyph_color(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    danger: bool,
) -> [u8; 4] {
    if is_unavailable_icon_button_state(state) {
        PALETTE.text_disabled
    } else if danger {
        PALETTE.error
    } else {
        match state {
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => PALETTE.focus_ring,
            UiPainterResolvedState::Hovered => PALETTE.text,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                PALETTE.text_disabled
            }
            UiPainterResolvedState::Normal => declared_icon_color(node).unwrap_or_else(|| {
                if context == WorkbenchIconButtonContext::Rail {
                    ICON_MUTED
                } else {
                    ICON_NORMAL
                }
            }),
        }
    }
}
