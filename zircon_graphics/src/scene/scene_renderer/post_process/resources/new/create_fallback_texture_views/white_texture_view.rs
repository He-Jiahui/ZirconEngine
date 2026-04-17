use super::super::super::super::fallback_texture::create_fallback_texture_view;

pub(super) fn white_texture_view(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    create_fallback_texture_view(device, queue, [255, 255, 255, 255], "zircon-white-fallback")
}
