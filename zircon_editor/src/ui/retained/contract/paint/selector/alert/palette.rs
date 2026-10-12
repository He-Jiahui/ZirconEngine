//! 从当前宿主主题提取警报语义容器和前景色，使选择器随主题快照更新；测试可注入固定调色板核对映射。

use super::model::{WorkbenchAlertStyle, WorkbenchAlertTone};
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchAlertPalette {
    pub info_surface: [u8; 4],
    pub info_tone: [u8; 4],
    pub success_surface: [u8; 4],
    pub success_tone: [u8; 4],
    pub warning_surface: [u8; 4],
    pub warning_tone: [u8; 4],
    pub error_surface: [u8; 4],
    pub error_tone: [u8; 4],
    pub text: [u8; 4],
    pub disabled_surface: [u8; 4],
    pub disabled_border: [u8; 4],
    pub disabled_text: [u8; 4],
}

pub(super) fn workbench_alert_palette() -> WorkbenchAlertPalette {
    workbench_alert_palette_from_host(current_host_palette())
}

pub(super) fn workbench_alert_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchAlertPalette {
    WorkbenchAlertPalette {
        info_surface: palette.info_container,
        info_tone: palette.info,
        success_surface: palette.success_container,
        success_tone: palette.success,
        warning_surface: palette.warning_container,
        warning_tone: palette.warning,
        error_surface: palette.error_container,
        error_tone: palette.error,
        text: palette.text,
        disabled_surface: palette.surface_disabled,
        disabled_border: palette.border_disabled,
        disabled_text: palette.text_disabled,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn alert_tone_style(
    tone: WorkbenchAlertTone,
    state: UiPainterResolvedState,
) -> WorkbenchAlertStyle {
    alert_tone_style_from_palette(tone, state, workbench_alert_palette())
}

#[cfg(test)]
pub(super) fn alert_tone_style_from_host(
    tone: WorkbenchAlertTone,
    state: UiPainterResolvedState,
    palette: HostMaterialPalette,
) -> WorkbenchAlertStyle {
    alert_tone_style_from_palette(tone, state, workbench_alert_palette_from_host(palette))
}

pub(super) fn alert_tone_style_from_palette(
    tone: WorkbenchAlertTone,
    state: UiPainterResolvedState,
    palette: WorkbenchAlertPalette,
) -> WorkbenchAlertStyle {
    let (surface, border, mark) = match tone {
        WorkbenchAlertTone::Info => alert_info_from_palette(palette),
        WorkbenchAlertTone::Success => alert_success_from_palette(palette),
        WorkbenchAlertTone::Warning => alert_warning_from_palette(palette),
        WorkbenchAlertTone::Error => alert_error_from_palette(palette),
    };
    WorkbenchAlertStyle {
        surface,
        border,
        mark,
        text: palette.text,
        state,
    }
}

fn alert_info_from_palette(palette: WorkbenchAlertPalette) -> ([u8; 4], [u8; 4], [u8; 4]) {
    (palette.info_surface, palette.info_tone, palette.info_tone)
}

fn alert_success_from_palette(palette: WorkbenchAlertPalette) -> ([u8; 4], [u8; 4], [u8; 4]) {
    (
        palette.success_surface,
        palette.success_tone,
        palette.success_tone,
    )
}

fn alert_warning_from_palette(palette: WorkbenchAlertPalette) -> ([u8; 4], [u8; 4], [u8; 4]) {
    (
        palette.warning_surface,
        palette.warning_tone,
        palette.warning_tone,
    )
}

fn alert_error_from_palette(palette: WorkbenchAlertPalette) -> ([u8; 4], [u8; 4], [u8; 4]) {
    (
        palette.error_surface,
        palette.error_tone,
        palette.error_tone,
    )
}

#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_ALERT_INFO_SURFACE: [u8; 4] =
    PALETTE.info_container;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_ALERT_WARNING_SURFACE: [u8; 4] =
    PALETTE.warning_container;

#[cfg(test)]
#[path = "tests/palette.rs"]
mod tests;
