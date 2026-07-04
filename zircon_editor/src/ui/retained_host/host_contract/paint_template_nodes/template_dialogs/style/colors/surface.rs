use super::super::palette::dialog_palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_surface_color(
    unavailable: bool,
) -> [u8; 4] {
    let palette = dialog_palette();
    if unavailable {
        palette.disabled_surface
    } else {
        palette.surface
    }
}
