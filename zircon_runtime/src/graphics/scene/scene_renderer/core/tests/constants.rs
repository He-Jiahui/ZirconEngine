use super::{FINAL_COLOR_FORMAT, SCENE_COLOR_HDR_FORMAT};

#[test]
fn scene_color_stays_linear_hdr_until_final_output_transfer() {
    assert_eq!(SCENE_COLOR_HDR_FORMAT, wgpu::TextureFormat::Rgba16Float);
    assert_eq!(FINAL_COLOR_FORMAT, wgpu::TextureFormat::Rgba8UnormSrgb);
    assert_ne!(SCENE_COLOR_HDR_FORMAT, FINAL_COLOR_FORMAT);
}
