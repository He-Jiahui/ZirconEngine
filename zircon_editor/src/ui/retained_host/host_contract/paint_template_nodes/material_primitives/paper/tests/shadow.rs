use super::*;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

#[test]
fn paper_shadow_layers_project_from_host_shadow_color() {
    let mut palette = PALETTE;
    palette.shadow = [10, 20, 30, 200];

    let layers = shadow_layers_from_host(9.0, palette);

    assert_eq!(layers[0].color, [10, 20, 30, 54]);
    assert_eq!(layers[1].color, [10, 20, 30, 62]);
    assert_eq!(layers[2].color, [10, 20, 30, 88]);
}

#[test]
fn paper_shadow_layers_keep_elevation_geometry() {
    let layers = shadow_layers_from_host(9.0, PALETTE);

    assert_eq!(layers[0].offset_y, 3.0);
    assert_eq!(layers[0].grow, 1.0);
    assert_eq!(layers[1].offset_y, 9.0);
    assert_eq!(layers[1].grow, 0.0);
    assert_eq!(layers[2].offset_y, 9.0);
    assert_eq!(layers[2].grow, 0.0);
}
