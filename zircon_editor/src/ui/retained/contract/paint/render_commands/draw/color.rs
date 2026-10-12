pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn color_with_opacity(
    mut color: [u8; 4],
    opacity: f32,
) -> [u8; 4] {
    let opacity = opacity.clamp(0.0, 1.0);
    color[3] = ((color[3] as f32 * opacity).round()).clamp(0.0, 255.0) as u8;
    color
}
