use super::model::WorkbenchTooltipStyle;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_SURFACE: [u8; 4] =
    PALETTE.popup;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_BORDER: [u8; 4] =
    PALETTE.border;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_TITLE: [u8; 4] =
    PALETTE.text;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_BODY: [u8; 4] =
    PALETTE.text_muted;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_ICON: [u8; 4] =
    PALETTE.focus_ring;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_SHADOW: [u8; 4] = PALETTE.shadow;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tooltip_normal_style(
    state: UiPainterResolvedState,
) -> WorkbenchTooltipStyle {
    WorkbenchTooltipStyle {
        surface: WORKBENCH_TOOLTIP_SURFACE,
        border: WORKBENCH_TOOLTIP_BORDER,
        title: WORKBENCH_TOOLTIP_TITLE,
        body: WORKBENCH_TOOLTIP_BODY,
        arrow: WORKBENCH_TOOLTIP_SURFACE,
        icon: WORKBENCH_TOOLTIP_ICON,
        shadow: WORKBENCH_TOOLTIP_SHADOW,
        state,
    }
}
