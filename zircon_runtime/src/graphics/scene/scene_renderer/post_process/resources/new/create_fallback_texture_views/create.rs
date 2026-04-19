use super::super::fallback_texture_views::FallbackTextureViews;
use super::black_texture_view::black_texture_view;
use super::white_texture_view::white_texture_view;

pub(in super::super) fn create_fallback_texture_views(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> FallbackTextureViews {
    FallbackTextureViews {
        black_texture_view: black_texture_view(device, queue),
        white_texture_view: white_texture_view(device, queue),
    }
}
