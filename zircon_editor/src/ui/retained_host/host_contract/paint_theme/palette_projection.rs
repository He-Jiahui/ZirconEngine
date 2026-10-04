use super::model::HostMaterialPalette;
use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;
use zircon_runtime_interface::ui::design_tokens::EditorPaletteTokens;
#[cfg(test)]
use zircon_runtime_interface::ui::style::UiRgbaColor;

pub(in crate::ui::retained_host::host_contract) const DEFAULT_HOST_PALETTE: HostMaterialPalette =
    default_host_palette_from_central_tokens();

pub(crate) fn apply_host_palette_from_tokens(tokens: &EditorDesignTokens) {
    super::replace_host_palette(project_host_palette(tokens));
}

pub(in crate::ui::retained_host::host_contract) fn current_host_palette() -> HostMaterialPalette {
    super::host_palette_for_read()
}

pub(in crate::ui::retained_host::host_contract) fn project_host_palette(
    tokens: &EditorDesignTokens,
) -> HostMaterialPalette {
    let palette = &tokens.palette;
    HostMaterialPalette {
        shell_background: palette.surface[0].to_u8(),
        surface: palette.surface[2].to_u8(),
        surface_inset: palette.surface_recessed.to_u8(),
        surface_input: palette.surface_input.to_u8(),
        surface_panel: palette.surface[1].to_u8(),
        surface_tab_hover: palette.surface_tab_hover.to_u8(),
        text_tab_active: palette.text_tab_active.to_u8(),
        surface_hover: palette.surface_hover.to_u8(),
        surface_pressed: palette.surface[3].to_u8(),
        surface_selected: palette.surface_selected.to_u8(),
        surface_disabled: palette.surface_disabled.to_u8(),
        accent: palette.accent.to_u8(),
        accent_soft: palette.accent_soft.to_u8(),
        border: palette.border.to_u8(),
        separator_strong: palette.separator_strong.to_u8(),
        separator_soft: palette.separator_soft.to_u8(),
        text: palette.text_primary.to_u8(),
        text_muted: palette.text_secondary.to_u8(),
        text_disabled: palette.text_disabled.to_u8(),
        warning: palette.warning.to_u8(),
        warning_container: palette.warning_container.to_u8(),
        error: palette.error.to_u8(),
        error_container: palette.error_container.to_u8(),
        success: palette.success.to_u8(),
        success_container: palette.success_container.to_u8(),
        info: palette.info.to_u8(),
        info_container: palette.info_container.to_u8(),
        popup: palette.popup.to_u8(),
        track: palette.track.to_u8(),
        focus_ring: palette.focus_ring.to_u8(),
        border_disabled: palette.border_disabled.to_u8(),
        shadow: palette.shadow.to_u8(),
    }
}

const fn default_host_palette_from_central_tokens() -> HostMaterialPalette {
    HostMaterialPalette {
        shell_background: EditorPaletteTokens::WORKBENCH_SURFACE[0],
        surface: EditorPaletteTokens::WORKBENCH_SURFACE[2],
        surface_inset: EditorPaletteTokens::WORKBENCH_SURFACE_RECESSED,
        surface_input: EditorPaletteTokens::WORKBENCH_SURFACE_INPUT,
        surface_panel: EditorPaletteTokens::WORKBENCH_SURFACE[1],
        surface_tab_hover: EditorPaletteTokens::WORKBENCH_SURFACE_TAB_HOVER,
        text_tab_active: EditorPaletteTokens::WORKBENCH_TEXT_TAB_ACTIVE,
        surface_hover: EditorPaletteTokens::WORKBENCH_SURFACE_HOVER,
        surface_pressed: EditorPaletteTokens::WORKBENCH_SURFACE[3],
        surface_selected: EditorPaletteTokens::WORKBENCH_SURFACE_SELECTED,
        surface_disabled: EditorPaletteTokens::WORKBENCH_SURFACE_DISABLED,
        accent: EditorPaletteTokens::WORKBENCH_ACCENT,
        accent_soft: EditorPaletteTokens::WORKBENCH_ACCENT_SOFT,
        border: EditorPaletteTokens::WORKBENCH_BORDER,
        separator_strong: EditorPaletteTokens::WORKBENCH_SEPARATOR_STRONG,
        separator_soft: EditorPaletteTokens::WORKBENCH_SEPARATOR_SOFT,
        text: EditorPaletteTokens::WORKBENCH_TEXT_PRIMARY,
        text_muted: EditorPaletteTokens::WORKBENCH_TEXT_SECONDARY,
        text_disabled: EditorPaletteTokens::WORKBENCH_TEXT_DISABLED,
        warning: EditorPaletteTokens::WORKBENCH_WARNING,
        warning_container: EditorPaletteTokens::WORKBENCH_WARNING_CONTAINER,
        error: EditorPaletteTokens::WORKBENCH_ERROR,
        error_container: EditorPaletteTokens::WORKBENCH_ERROR_CONTAINER,
        success: EditorPaletteTokens::WORKBENCH_SUCCESS,
        success_container: EditorPaletteTokens::WORKBENCH_SUCCESS_CONTAINER,
        info: EditorPaletteTokens::WORKBENCH_INFO,
        info_container: EditorPaletteTokens::WORKBENCH_INFO_CONTAINER,
        popup: EditorPaletteTokens::WORKBENCH_POPUP,
        track: EditorPaletteTokens::WORKBENCH_TRACK,
        focus_ring: EditorPaletteTokens::WORKBENCH_FOCUS_RING,
        border_disabled: EditorPaletteTokens::WORKBENCH_BORDER_DISABLED,
        shadow: EditorPaletteTokens::WORKBENCH_SHADOW,
    }
}

#[cfg(test)]
#[path = "tests/palette_projection.rs"]
mod tests;
