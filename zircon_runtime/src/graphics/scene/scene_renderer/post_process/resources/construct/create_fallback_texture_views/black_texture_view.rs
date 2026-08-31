use crate::graphics::backend::SystemTextureGenerationLease;

pub(super) fn black_texture_view(
    system_textures: &SystemTextureGenerationLease,
) -> wgpu::TextureView {
    system_textures.black_alpha_one_rgba8_view().clone()
}
