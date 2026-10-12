use super::{
    transmission_scene_color_bind_group_layout_entries, GpuTransmissionSceneColorParams,
    TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL,
};

#[test]
fn render_transmission_zero_step_fallback_marks_scene_copy_unavailable() {
    assert_eq!(TRANSMISSION_SCENE_COLOR_FALLBACK_TEXEL, [0, 0, 0, 0]);
    assert_eq!(GpuTransmissionSceneColorParams::new(false).available, 0);
    assert_eq!(GpuTransmissionSceneColorParams::new(true).available, 1);
    assert_eq!(std::mem::size_of::<GpuTransmissionSceneColorParams>(), 16);
    let layout_entries = transmission_scene_color_bind_group_layout_entries();
    assert_eq!(
        layout_entries.each_ref().map(|entry| entry.binding),
        [31, 32, 38]
    );
    let wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        min_binding_size,
        ..
    } = &layout_entries[2].ty
    else {
        panic!("transmission scene-color params must remain a uniform buffer");
    };
    assert_eq!(min_binding_size.as_ref().map(|size| size.get()), Some(16));
}
