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
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

    #[test]
    fn toast_normal_style_projects_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent_soft = [10, 11, 12, 247];
        palette.border = [13, 14, 15, 20];
        palette.text = [16, 17, 18, 255];
        palette.accent = [19, 20, 21, 255];
        palette.text_muted = [22, 23, 24, 255];

        let style = toast_normal_style_from_host(UiPainterResolvedState::Normal, palette);

        assert_eq!(style.surface, [10, 11, 12, 247]);
        assert_eq!(style.border, [13, 14, 15, 20]);
        assert_eq!(style.text, [16, 17, 18, 255]);
        assert_eq!(style.mark, [19, 20, 21, 255]);
        assert_eq!(style.action, [19, 20, 21, 255]);
        assert_eq!(style.close, [22, 23, 24, 255]);
    }

    #[test]
    fn toast_interaction_surfaces_project_from_host_palette() {
        let mut palette = PALETTE;
        palette.surface_selected = [30, 31, 32, 255];
        palette.surface_pressed = [33, 34, 35, 255];

        assert_eq!(toast_hover_surface_from_host(palette), [30, 31, 32, 255]);
        assert_eq!(toast_pressed_surface_from_host(palette), [33, 34, 35, 255]);
    }

    #[test]
    fn toast_palette_projects_state_roles_from_host_palette() {
        let mut palette = PALETTE;
        palette.accent_soft = [40, 41, 42, 247];
        palette.focus_ring = [41, 42, 43, 255];
        palette.surface_disabled = [43, 44, 45, 255];
        palette.border_disabled = [46, 47, 48, 255];
        palette.text_disabled = [49, 50, 51, 255];

        let toast_palette = workbench_toast_palette_from_host(palette);

        assert_eq!(toast_palette.hover_border, [40, 41, 42, 247]);
        assert_eq!(toast_palette.focus_border, [41, 42, 43, 255]);
        assert_eq!(toast_palette.disabled_surface, [43, 44, 45, 255]);
        assert_eq!(toast_palette.disabled_border, [46, 47, 48, 255]);
        assert_eq!(toast_palette.disabled_text, [49, 50, 51, 255]);
    }
}
