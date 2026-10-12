use crate::graphics::backend::SystemTextureGenerationLease;

pub(super) fn effect_lut_texture_view(
    system_textures: &SystemTextureGenerationLease,
) -> wgpu::TextureView {
    system_textures.effect_lut_view().clone()
}

pub(super) fn effect_lut_texture_3d_view(
    system_textures: &SystemTextureGenerationLease,
) -> wgpu::TextureView {
    system_textures.effect_lut_3d_view().clone()
}
