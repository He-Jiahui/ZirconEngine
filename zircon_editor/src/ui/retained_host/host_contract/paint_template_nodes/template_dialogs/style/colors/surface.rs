use super::super::tokens::{DIALOG_DISABLED_SURFACE, DIALOG_SURFACE};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_surface_color(
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_SURFACE
    } else {
        DIALOG_SURFACE
    }
}
