use super::model::{WorkbenchAlertStyle, WorkbenchAlertTone};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_ALERT_INFO_SURFACE: [u8; 4] =
    PALETTE.info_container;
const WORKBENCH_ALERT_INFO_BORDER: [u8; 4] = PALETTE.info;
const WORKBENCH_ALERT_SUCCESS_SURFACE: [u8; 4] = PALETTE.success_container;
const WORKBENCH_ALERT_SUCCESS_BORDER: [u8; 4] = PALETTE.success;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_ALERT_WARNING_SURFACE: [u8; 4] =
    PALETTE.warning_container;
const WORKBENCH_ALERT_WARNING_BORDER: [u8; 4] = PALETTE.warning;
const WORKBENCH_ALERT_ERROR_SURFACE: [u8; 4] = PALETTE.error_container;
const WORKBENCH_ALERT_ERROR_BORDER: [u8; 4] = PALETTE.error;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_tone_style(
    tone: WorkbenchAlertTone,
    state: UiPainterResolvedState,
) -> WorkbenchAlertStyle {
    let (surface, border, mark) = match tone {
        WorkbenchAlertTone::Info => (
            WORKBENCH_ALERT_INFO_SURFACE,
            WORKBENCH_ALERT_INFO_BORDER,
            PALETTE.info,
        ),
        WorkbenchAlertTone::Success => (
            WORKBENCH_ALERT_SUCCESS_SURFACE,
            WORKBENCH_ALERT_SUCCESS_BORDER,
            PALETTE.success,
        ),
        WorkbenchAlertTone::Warning => (
            WORKBENCH_ALERT_WARNING_SURFACE,
            WORKBENCH_ALERT_WARNING_BORDER,
            PALETTE.warning,
        ),
        WorkbenchAlertTone::Error => (
            WORKBENCH_ALERT_ERROR_SURFACE,
            WORKBENCH_ALERT_ERROR_BORDER,
            PALETTE.error,
        ),
    };
    WorkbenchAlertStyle {
        surface,
        border,
        mark,
        text: mark,
        state,
    }
}
