use super::model::WorkbenchToastStyle;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_SURFACE: [u8; 4] =
    PALETTE.accent_soft;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_BORDER: [u8; 4] =
    PALETTE.border;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_TEXT:
    [u8; 4] = PALETTE.text;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_ACTION: [u8; 4] =
    PALETTE.focus_ring;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_HOVER_SURFACE: [u8; 4] = PALETTE.surface_selected;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_PRESSED_SURFACE: [u8; 4] = PALETTE.surface_pressed;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_normal_style(
    state: UiPainterResolvedState,
) -> WorkbenchToastStyle {
    WorkbenchToastStyle {
        surface: WORKBENCH_TOAST_SURFACE,
        border: WORKBENCH_TOAST_BORDER,
        text: WORKBENCH_TOAST_TEXT,
        mark: WORKBENCH_TOAST_ACTION,
        action: WORKBENCH_TOAST_ACTION,
        close: PALETTE.text_muted,
        state,
    }
}
