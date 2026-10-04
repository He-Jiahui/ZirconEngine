use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

use super::*;

#[test]
fn axis_label_palette_keeps_audited_default_tones_from_host_palette() {
    let palette = axis_label_palette_from_host(PALETTE);

    assert_eq!(palette.axis, [129, 136, 140, 255]);
    assert_eq!(palette.scale_axis, [126, 132, 136, 255]);
    assert_eq!(palette.scale_link, [145, 157, 164, 255]);
    assert_eq!(palette.disabled_axis, [82, 93, 100, 255]);
    assert_eq!(palette.disabled_scale_link, [82, 93, 100, 255]);
}

#[test]
fn axis_label_palette_projects_from_host_material_palette() {
    let mut host = PALETTE;
    host.text_muted = [200, 150, 100, 180];
    host.text_disabled = [80, 90, 100, 170];

    let palette = axis_label_palette_from_host(host);

    assert_eq!(palette.axis, [157, 117, 78, 180]);
    assert_eq!(palette.scale_axis, [154, 114, 76, 180]);
    assert_eq!(palette.scale_link, [177, 135, 91, 180]);
    assert_eq!(palette.disabled_axis, [65, 75, 85, 170]);
    assert_eq!(palette.disabled_scale_link, [65, 75, 85, 170]);
}
