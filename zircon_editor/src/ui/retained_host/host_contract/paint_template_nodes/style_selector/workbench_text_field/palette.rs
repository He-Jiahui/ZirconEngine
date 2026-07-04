#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_SURFACE: [u8; 4] =
    PALETTE.surface_inset;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE: [u8; 4] =
    PALETTE.surface;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_DISABLED_SURFACE: [u8; 4] =
    PALETTE.surface;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_BORDER: [u8; 4] =
    PALETTE.separator_soft;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_FOCUSED_BORDER: [u8; 4] =
    PALETTE.border;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_DISABLED_BORDER: [u8; 4] =
    PALETTE.border_disabled;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_PLACEHOLDER: [u8; 4] =
    PALETTE.text_muted;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_DISABLED_TEXT: [u8; 4] =
    PALETTE.text_disabled;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_STEPPER_DIVIDER: [u8; 4] =
    PALETTE.separator_soft;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorkbenchTextFieldPalette {
    pub surface: [u8; 4],
    pub toolbar_surface: [u8; 4],
    pub hover_surface: [u8; 4],
    pub focused_surface: [u8; 4],
    pub disabled_surface: [u8; 4],
    pub border: [u8; 4],
    pub focused_border: [u8; 4],
    pub disabled_border: [u8; 4],
    pub text: [u8; 4],
    pub placeholder: [u8; 4],
    pub disabled_text: [u8; 4],
    pub stepper_divider: [u8; 4],
    pub error: [u8; 4],
    pub focus_ring: [u8; 4],
}

pub(super) fn workbench_text_field_palette() -> WorkbenchTextFieldPalette {
    workbench_text_field_palette_from_host(current_host_palette())
}

pub(super) fn workbench_text_field_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchTextFieldPalette {
    WorkbenchTextFieldPalette {
        surface: palette.surface_inset,
        toolbar_surface: palette.surface,
        hover_surface: palette.surface,
        focused_surface: palette.surface,
        disabled_surface: palette.surface,
        border: palette.separator_soft,
        focused_border: palette.border,
        disabled_border: palette.border_disabled,
        text: palette.text,
        placeholder: palette.text_muted,
        disabled_text: palette.text_disabled,
        stepper_divider: palette.separator_soft,
        error: palette.error,
        focus_ring: palette.focus_ring,
    }
}
