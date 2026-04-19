#[derive(Clone, Debug)]
pub struct ViewportFrameTextureHandle {
    pub width: u32,
    pub height: u32,
    pub texture: wgpu::Texture,
    pub format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
    pub generation: u64,
}
