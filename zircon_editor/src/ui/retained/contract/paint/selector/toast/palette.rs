//! 当前宿主主题的 Toast 角色投影；柔和强调表面、可见焦点边线及操作强调色分别承担不同视觉含义。
//! 文件内 cfg(test) 投影测试验证角色映射；运行时入口读取当前主题。

use super::model::WorkbenchToastStyle;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchToastPalette {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub focus_border: [u8; 4],
    pub text: [u8; 4],
    pub action: [u8; 4],
    pub close: [u8; 4],
    pub hover_surface: [u8; 4],
    pub hover_border: [u8; 4],
    pub pressed_surface: [u8; 4],
    pub disabled_surface: [u8; 4],
    pub disabled_border: [u8; 4],
    pub disabled_text: [u8; 4],
}

pub(super) fn workbench_toast_palette() -> WorkbenchToastPalette {
    workbench_toast_palette_from_host(current_host_palette())
}

pub(super) fn workbench_toast_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchToastPalette {
    WorkbenchToastPalette {
        surface: palette.accent_soft,
        border: palette.border,
        focus_border: palette.focus_ring,
        text: palette.text,
        action: palette.accent,
        close: palette.text_muted,
        hover_surface: palette.surface_selected,
        hover_border: palette.accent_soft,
        pressed_surface: palette.surface_pressed,
        disabled_surface: palette.surface_disabled,
        disabled_border: palette.border_disabled,
        disabled_text: palette.text_disabled,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_normal_style(
    state: UiPainterResolvedState,
) -> WorkbenchToastStyle {
    toast_normal_style_from_palette(state, workbench_toast_palette())
}

#[cfg(test)]
pub(super) fn toast_normal_style_from_host(
    state: UiPainterResolvedState,
    palette: HostMaterialPalette,
) -> WorkbenchToastStyle {
    toast_normal_style_from_palette(state, workbench_toast_palette_from_host(palette))
}

pub(super) fn toast_normal_style_from_palette(
    state: UiPainterResolvedState,
    palette: WorkbenchToastPalette,
) -> WorkbenchToastStyle {
    WorkbenchToastStyle {
        surface: palette.surface,
        border: palette.border,
        text: palette.text,
        mark: palette.action,
        action: palette.action,
        close: palette.close,
        state,
    }
}

#[cfg(test)]
pub(super) fn toast_surface_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    workbench_toast_palette_from_host(palette).surface
}

#[cfg(test)]
pub(super) fn toast_border_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    workbench_toast_palette_from_host(palette).border
}

#[cfg(test)]
fn toast_text_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    workbench_toast_palette_from_host(palette).text
}

#[cfg(test)]
pub(super) fn toast_action_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    workbench_toast_palette_from_host(palette).action
}

#[cfg(test)]
fn toast_close_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    workbench_toast_palette_from_host(palette).close
}

#[cfg(test)]
pub(super) fn toast_hover_surface_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    workbench_toast_palette_from_host(palette).hover_surface
}

#[cfg(test)]
pub(super) fn toast_pressed_surface_from_host(palette: HostMaterialPalette) -> [u8; 4] {
    workbench_toast_palette_from_host(palette).pressed_surface
}

#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_SURFACE: [u8; 4] =
    PALETTE.accent_soft;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_BORDER: [u8; 4] =
    PALETTE.border;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOAST_ACTION: [u8; 4] =
    PALETTE.accent;

#[cfg(test)]
#[path = "tests/palette.rs"]
mod tests;
