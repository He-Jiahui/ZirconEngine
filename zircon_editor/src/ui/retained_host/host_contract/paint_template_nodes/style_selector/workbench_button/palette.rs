use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

const TRANSPARENT_SURFACE: [u8; 4] = [0, 0, 0, 0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchButtonPalette
{
    pub surface_base: [u8; 4],
    pub surface_primary_rest: [u8; 4],
    pub surface_primary_hover: [u8; 4],
    pub surface_hover: [u8; 4],
    pub surface_primary_pressed: [u8; 4],
    pub surface_secondary_pressed: [u8; 4],
    pub surface_tertiary_pressed: [u8; 4],
    pub surface_danger_pressed: [u8; 4],
    pub transparent_surface: [u8; 4],
    pub border: [u8; 4],
    pub focus_border: [u8; 4],
    pub primary_text: [u8; 4],
    pub primary_pressed_text: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub danger_text: [u8; 4],
    pub disabled_surface: [u8; 4],
    pub disabled_border: [u8; 4],
    pub disabled_text: [u8; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchButtonCommandPalette
{
    pub muted_rest_surface: [u8; 4],
    pub muted_hot_surface: [u8; 4],
    pub muted_pressed_surface: [u8; 4],
    pub muted_border: [u8; 4],
    pub muted_text: [u8; 4],
    pub primary_rest_surface: [u8; 4],
    pub primary_hot_surface: [u8; 4],
    pub primary_pressed_surface: [u8; 4],
    pub primary_text: [u8; 4],
    pub primary_pressed_text: [u8; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchButtonSelectionPalette
{
    pub tab_rest_surface: [u8; 4],
    pub tab_hot_surface: [u8; 4],
    pub toolbar_chip_active_surface: [u8; 4],
    pub asset_tab_active_surface: [u8; 4],
    pub transparent_surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn add_component_text_color(
) -> [u8; 4] {
    add_component_text_color_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn add_component_glyph_color(
) -> [u8; 4] {
    add_component_glyph_color_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_transparent_surface(
) -> [u8; 4] {
    TRANSPARENT_SURFACE
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_palette(
) -> WorkbenchButtonPalette {
    workbench_button_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_command_palette(
) -> WorkbenchButtonCommandPalette {
    workbench_button_command_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_selection_palette(
) -> WorkbenchButtonSelectionPalette {
    workbench_button_selection_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchButtonPalette {
    WorkbenchButtonPalette {
        surface_base: palette.surface_pressed,
        // Mirrors Starship's PrimaryButton fill and inverted-foreground hierarchy.
        surface_primary_rest: palette.accent,
        surface_primary_hover: palette.focus_ring,
        surface_hover: palette.surface_hover,
        surface_primary_pressed: palette.surface_selected,
        surface_secondary_pressed: palette.surface,
        surface_tertiary_pressed: palette.popup,
        surface_danger_pressed: palette.surface,
        transparent_surface: TRANSPARENT_SURFACE,
        border: palette.border,
        focus_border: palette.focus_ring,
        primary_text: palette.shell_background,
        primary_pressed_text: palette.text,
        text: palette.text,
        text_muted: palette.text_muted,
        danger_text: palette.error,
        disabled_surface: palette.surface_disabled,
        disabled_border: palette.border_disabled,
        disabled_text: palette.text_disabled,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_command_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchButtonCommandPalette {
    WorkbenchButtonCommandPalette {
        muted_rest_surface: palette.surface_pressed,
        muted_hot_surface: palette.surface_hover,
        muted_pressed_surface: palette.surface,
        muted_border: palette.border,
        muted_text: palette.accent,
        primary_rest_surface: palette.accent,
        primary_hot_surface: palette.focus_ring,
        primary_pressed_surface: palette.surface_selected,
        primary_text: palette.shell_background,
        primary_pressed_text: palette.text,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_button_selection_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchButtonSelectionPalette {
    WorkbenchButtonSelectionPalette {
        tab_rest_surface: palette.surface_pressed,
        tab_hot_surface: palette.surface_hover,
        toolbar_chip_active_surface: palette.surface,
        asset_tab_active_surface: palette.surface_pressed,
        transparent_surface: TRANSPARENT_SURFACE,
        border: palette.border,
        text: palette.text,
        text_muted: palette.text_muted,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn add_component_text_color_from_host(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.text_muted
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn add_component_glyph_color_from_host(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.text
}
