use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn mui_x_picker_popup_colors_project_from_host_palette() {
    let mut palette = PALETTE;
    palette.surface = [10, 11, 12, 255];
    palette.accent_soft = [20, 21, 22, 255];

    assert_eq!(
        picker_popup_colors_from_host(palette),
        [[10, 11, 12, 255], [20, 21, 22, 255], [20, 21, 22, 255],]
    );
}
