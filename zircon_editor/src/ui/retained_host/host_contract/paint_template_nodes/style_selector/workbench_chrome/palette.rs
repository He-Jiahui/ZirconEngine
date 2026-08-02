#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_ROOT_BG: [u8; 4] = PALETTE.shell_background;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_TOPBAR_BG:
    [u8; 4] = PALETTE.shell_background;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_MAIN_BG: [u8; 4] = PALETTE.shell_background;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_RAIL_BG: [u8; 4] = PALETTE.surface_inset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_PANEL_BG:
    [u8; 4] = PALETTE.surface;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_CONTENT_BG:
    [u8; 4] = PALETTE.surface_inset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_VIEWPORT_FRAME_BG: [u8; 4] = PALETTE.surface_inset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_DRAWER_BG:
    [u8; 4] = PALETTE.surface;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_DRAWER_BODY_BG: [u8; 4] = PALETTE.surface_inset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_STATUS_BG:
    [u8; 4] = PALETTE.shell_background;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_TAB_BG: [u8; 4] = PALETTE.surface_pressed;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_SEPARATOR: [u8; 4] = PALETTE.border;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_STRONG_SEPARATOR:
    [u8; 4] = PALETTE.separator_strong;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_CHROME_SOFT_SEPARATOR:
    [u8; 4] = PALETTE.separator_soft;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchChromePalette {
    pub root_bg: [u8; 4],
    pub topbar_bg: [u8; 4],
    pub main_bg: [u8; 4],
    pub rail_bg: [u8; 4],
    pub panel_bg: [u8; 4],
    pub content_bg: [u8; 4],
    pub viewport_frame_bg: [u8; 4],
    pub drawer_bg: [u8; 4],
    pub drawer_body_bg: [u8; 4],
    pub status_bg: [u8; 4],
    pub tab_bg: [u8; 4],
    pub separator: [u8; 4],
    pub strong_separator: [u8; 4],
    pub soft_separator: [u8; 4],
    pub surface_disabled: [u8; 4],
    pub surface_pressed: [u8; 4],
    pub surface_hover: [u8; 4],
    pub surface_selected: [u8; 4],
    pub border: [u8; 4],
    pub border_disabled: [u8; 4],
}

pub(super) fn workbench_chrome_palette() -> WorkbenchChromePalette {
    workbench_chrome_palette_from_host(current_host_palette())
}

pub(super) fn workbench_chrome_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchChromePalette {
    WorkbenchChromePalette {
        root_bg: palette.shell_background,
        topbar_bg: palette.shell_background,
        main_bg: palette.shell_background,
        rail_bg: palette.surface_inset,
        panel_bg: palette.surface,
        content_bg: palette.surface_inset,
        viewport_frame_bg: palette.surface_inset,
        drawer_bg: palette.surface,
        drawer_body_bg: palette.surface_inset,
        status_bg: palette.shell_background,
        tab_bg: palette.surface_pressed,
        separator: palette.border,
        strong_separator: palette.separator_strong,
        soft_separator: palette.separator_soft,
        surface_disabled: palette.surface_disabled,
        surface_pressed: palette.surface_pressed,
        surface_hover: palette.surface_hover,
        surface_selected: palette.surface_selected,
        border: palette.border,
        border_disabled: palette.border_disabled,
    }
}
