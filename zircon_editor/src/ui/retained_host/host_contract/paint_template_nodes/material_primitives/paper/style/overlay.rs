pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_dark_overlay(
    elevation: f32,
) -> [u8; 4] {
    let alpha = get_overlay_alpha(elevation);
    [255, 255, 255, (alpha * 255.0).round() as u8]
}

fn get_overlay_alpha(elevation: f32) -> f32 {
    if elevation < 1.0 {
        5.11916 * elevation.powi(2)
    } else {
        let alpha_value = 4.5 * (elevation + 1.0).ln() + 2.0;
        (alpha_value * 10.0).round() / 1000.0
    }
}
