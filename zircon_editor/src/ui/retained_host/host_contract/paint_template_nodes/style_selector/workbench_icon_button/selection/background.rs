use super::super::model::WorkbenchIconButtonContext;
use super::super::palette::ICON_PANEL_SURFACE;
use super::super::state::is_unavailable_icon_button_state;
use super::declared::declared_icon_button_background;
use super::toolbar_chrome::icon_toolbar_normal_chrome_enabled;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_background(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    danger: bool,
) -> Option<[u8; 4]> {
    if is_unavailable_icon_button_state(state) {
        return (context == WorkbenchIconButtonContext::Panel).then_some(PALETTE.surface_disabled);
    }
    if danger && context == WorkbenchIconButtonContext::Panel {
        return Some(PALETTE.error_container);
    }
    match state {
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            Some(PALETTE.surface_selected)
        }
        UiPainterResolvedState::Pressed => Some(PALETTE.surface_pressed),
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => Some(PALETTE.surface_hover),
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            (context == WorkbenchIconButtonContext::Panel).then_some(PALETTE.surface_disabled)
        }
        UiPainterResolvedState::Normal => {
            if context == WorkbenchIconButtonContext::Panel {
                declared_icon_button_background(node).or(Some(ICON_PANEL_SURFACE))
            } else if context == WorkbenchIconButtonContext::Toolbar
                && icon_toolbar_normal_chrome_enabled(node)
            {
                Some(PALETTE.surface)
            } else {
                None
            }
        }
    }
}
