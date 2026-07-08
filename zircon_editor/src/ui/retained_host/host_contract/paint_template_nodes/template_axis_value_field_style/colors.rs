use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_palette(
) -> HostMaterialPalette {
    current_host_palette()
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_normal_background(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.surface_inset
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_hover_background(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.surface_hover
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_pressed_background(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.surface_selected
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_disabled_background(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.surface_disabled
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_normal_border(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.border
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_hover_border(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.separator_strong
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_disabled_border(
    palette: HostMaterialPalette,
) -> [u8; 4] {
    palette.border_disabled
}
