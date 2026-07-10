pub(crate) const SCENE_COLOR_HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub(crate) const FINAL_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub(crate) const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[cfg(test)]
mod tests {
    use super::{FINAL_COLOR_FORMAT, SCENE_COLOR_HDR_FORMAT};

    #[test]
    fn scene_color_stays_linear_hdr_until_final_output_transfer() {
        assert_eq!(SCENE_COLOR_HDR_FORMAT, wgpu::TextureFormat::Rgba16Float);
        assert_eq!(FINAL_COLOR_FORMAT, wgpu::TextureFormat::Rgba8UnormSrgb);
        assert_ne!(SCENE_COLOR_HDR_FORMAT, FINAL_COLOR_FORMAT);
    }
}
