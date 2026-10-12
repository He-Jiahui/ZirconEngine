use super::super::fallback_texture_views::FallbackTextureViews;
use super::black_texture_view::black_texture_view;
use super::effect_lut_texture_view::{effect_lut_texture_3d_view, effect_lut_texture_view};
use super::hzb_source_texture_view::hzb_source_texture_view;
use super::white_texture_view::white_texture_view;
use crate::graphics::backend::SystemTextureGenerationLease;

/// 从同一系统纹理代际取得效果停用时的占位视图，使完整 bind group 始终可构造。
/// 构造时应使用当前设备的纹理租约；设备重建后，调用方须重新取得同设备视图。
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
