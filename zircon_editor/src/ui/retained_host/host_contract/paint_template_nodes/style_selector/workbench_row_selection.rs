use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;

pub(super) fn selected_row_outline_color() -> [u8; 4] {
    current_host_palette().border
}
