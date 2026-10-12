use crate::graphics::backend::SystemTextureGenerationLease;

pub(super) fn white_texture_view(
    system_textures: &SystemTextureGenerationLease,
) -> wgpu::TextureView {
    system_textures.white_rgba8_view().clone()
}
