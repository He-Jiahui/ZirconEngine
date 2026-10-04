use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn menu_popup_palette_uses_the_popup_and_border_theme_roles() {
    let mut palette = PALETTE;
    palette.popup = [18, 26, 35, 255];
    palette.border = [67, 89, 101, 255];

    let projected = menu_popup_palette(palette);

    assert_eq!(projected.surface, palette.popup);
    assert_eq!(projected.border, palette.border);
}
