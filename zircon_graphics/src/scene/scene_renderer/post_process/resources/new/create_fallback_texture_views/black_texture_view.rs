use super::super::super::super::fallback_texture::create_fallback_texture_view;

pub(super) fn black_texture_view(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    create_fallback_texture_view(device, queue, [0, 0, 0, 255], "zircon-black-fallback")
}
