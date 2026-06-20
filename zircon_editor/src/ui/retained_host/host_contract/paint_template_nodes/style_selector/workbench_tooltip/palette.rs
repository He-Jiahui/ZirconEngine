use super::model::WorkbenchTooltipStyle;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_SURFACE: [u8; 4] =
    [23, 28, 32, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_BORDER: [u8; 4] =
    [37, 45, 50, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_TITLE: [u8; 4] =
    [208, 217, 221, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_BODY: [u8; 4] =
    [168, 179, 184, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_ICON: [u8; 4] =
    [37, 156, 167, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_SHADOW: [u8; 4] = [0, 0, 0, 96];

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
