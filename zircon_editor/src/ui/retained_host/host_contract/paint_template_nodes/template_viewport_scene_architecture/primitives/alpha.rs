pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn color_with_alpha_factor(
    mut color: [u8; 4],
    factor: f32,
) -> [u8; 4] {
    color[3] = ((color[3] as f32) * factor).round().clamp(0.0, 255.0) as u8;
    color
}
