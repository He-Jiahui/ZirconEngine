use super::super::fallback_texture_views::FallbackTextureViews;
use super::black_texture_view::black_texture_view;
use super::effect_lut_texture_view::{effect_lut_texture_3d_view, effect_lut_texture_view};
use super::hzb_source_texture_view::hzb_source_texture_view;
use super::white_texture_view::white_texture_view;
use crate::graphics::backend::SystemTextureGenerationLease;

pub(in super::super) fn create_fallback_texture_views(
    system_textures: &SystemTextureGenerationLease,
) -> FallbackTextureViews {
    FallbackTextureViews {
        black_texture_view: black_texture_view(system_textures),
        white_texture_view: white_texture_view(system_textures),
        hzb_source_texture_view: hzb_source_texture_view(system_textures),
        effect_lut_texture_view: effect_lut_texture_view(system_textures),
        effect_lut_texture_3d_view: effect_lut_texture_3d_view(system_textures),
    }
}
