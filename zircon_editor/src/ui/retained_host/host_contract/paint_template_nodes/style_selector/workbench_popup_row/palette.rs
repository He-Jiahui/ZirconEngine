#[cfg(test)]
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_palette, HostMaterialPalette,
};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_POPUP_ROW_DANGER_TEXT:
    [u8; 4] = PALETTE.error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchPopupRowPalette
{
    pub marked_background: [u8; 4],
    pub hot_background: [u8; 4],
    pub text: [u8; 4],
    pub text_muted: [u8; 4],
    pub text_disabled: [u8; 4],
    pub danger_text: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_popup_row_palette(
) -> WorkbenchPopupRowPalette {
    workbench_popup_row_palette_from_host(current_host_palette())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_popup_row_palette_from_host(
    palette: HostMaterialPalette,
) -> WorkbenchPopupRowPalette {
    WorkbenchPopupRowPalette {
        marked_background: palette.surface_pressed,
        hot_background: palette.surface_hover,
        text: palette.text,
        text_muted: palette.text_muted,
        text_disabled: palette.text_disabled,
        danger_text: palette.error,
    }
}
