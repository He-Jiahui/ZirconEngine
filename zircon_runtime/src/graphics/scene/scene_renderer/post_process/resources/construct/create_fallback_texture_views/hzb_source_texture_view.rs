use crate::graphics::backend::SystemTextureGenerationLease;

pub(super) fn hzb_source_texture_view(
    system_textures: &SystemTextureGenerationLease,
) -> wgpu::TextureView {
    system_textures.black_rgba16float_view().clone()
}
