use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn draw_text_fallback_color_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.text = [10, 11, 12, 255];

    assert_eq!(fallback_text_from_host(palette), [10, 11, 12, 255]);
}
