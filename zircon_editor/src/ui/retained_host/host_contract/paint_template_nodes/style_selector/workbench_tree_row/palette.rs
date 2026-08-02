#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    HostMaterialPalette, current_host_palette,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TREE_ROW_TEXT_SELECTED: [u8; 4] =
    PALETTE.text;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTreeRowPalette
{
    pub marked_surface: [u8; 4],
    pub marked_hot_surface: [u8; 4],
    pub pressed_surface: [u8; 4],
    pub hot_surface: [u8; 4],
    pub focus_border: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub text_disabled: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_tree_row_palette()
-> WorkbenchTreeRowPalette {
    workbench_tree_row_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_tree_row_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchTreeRowPalette {
    WorkbenchTreeRowPalette {
        marked_surface: palette.surface_selected,
        marked_hot_surface: palette.accent_soft,
        pressed_surface: palette.surface_pressed,
        hot_surface: palette.surface_hover,
        focus_border: palette.focus_ring,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
    }
}
