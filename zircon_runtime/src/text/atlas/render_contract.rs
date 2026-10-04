//! 统一 CPU 批次、GPU 管线与 WGSL 对字形像素的解释。
//! 颜色与子像素覆盖率虽同用 RGBA 存储，解码和混合语义不同，不能只按纹理存储格式选择管线。

use super::{GlyphAtlasPageSpec, GlyphAtlasSamplingSemantics};

pub(crate) const GLYPH_ATLAS_SAMPLING_SHADER: &str =
    include_str!("shaders/glyph_atlas_sampling.wgsl");
pub(crate) const GLYPH_ATLAS_PIPELINE_SHADER: &str =
    include_str!("shaders/glyph_atlas_pipeline.wgsl");
pub(crate) const GLYPH_ATLAS_TEXT_SHADER: &str = concat!(
    include_str!("shaders/glyph_atlas_sampling.wgsl"),
    "\n",
    include_str!("shaders/glyph_atlas_pipeline.wgsl")
);

const GLYPH_ATLAS_VERTEX_ENTRY_POINT: &str = "vs_main";
const GLYPH_ATLAS_ALPHA_FRAGMENT_ENTRY_POINT: &str = "fs_alpha_coverage";
const GLYPH_ATLAS_SUBPIXEL_FRAGMENT_ENTRY_POINT: &str = "fs_subpixel_rgb_coverage";
const GLYPH_ATLAS_SIGNED_DISTANCE_FRAGMENT_ENTRY_POINT: &str = "fs_signed_distance_coverage";
const GLYPH_ATLAS_MSDF_FRAGMENT_ENTRY_POINT: &str = "fs_multi_channel_signed_distance_coverage";
const GLYPH_ATLAS_COLOR_FRAGMENT_ENTRY_POINT: &str = "fs_color_rgba";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasShaderDecode {
    AlphaCoverage,
    SubpixelRgbCoverage,
    SignedDistanceCoverage,
    MultiChannelSignedDistanceCoverage,
    ColorRgba,
}

impl GlyphAtlasShaderDecode {
    pub(crate) fn fragment_entry_point(self) -> &'static str {
        match self {
            Self::AlphaCoverage => GLYPH_ATLAS_ALPHA_FRAGMENT_ENTRY_POINT,
            Self::SubpixelRgbCoverage => GLYPH_ATLAS_SUBPIXEL_FRAGMENT_ENTRY_POINT,
            Self::SignedDistanceCoverage => GLYPH_ATLAS_SIGNED_DISTANCE_FRAGMENT_ENTRY_POINT,
            Self::MultiChannelSignedDistanceCoverage => GLYPH_ATLAS_MSDF_FRAGMENT_ENTRY_POINT,
            Self::ColorRgba => GLYPH_ATLAS_COLOR_FRAGMENT_ENTRY_POINT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasShaderEntryPoints {
    pub(crate) vertex: &'static str,
    pub(crate) fragment: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasBlendMode {
    StandardAlpha,
    SubpixelBackgroundComposite,
    SourceRgba,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 绘制批次必须携带的采样、解码和混合组合。
/// 子像素路径需要已知不透明背景，覆盖率路径用前景 alpha，颜色字形保留自身 RGBA。
pub(crate) struct GlyphAtlasRenderContract {
    pub(crate) sampling_semantics: GlyphAtlasSamplingSemantics,
    pub(crate) shader_decode: GlyphAtlasShaderDecode,
    pub(crate) blend_mode: GlyphAtlasBlendMode,
}

impl GlyphAtlasRenderContract {
    pub(crate) fn for_page(page: &GlyphAtlasPageSpec) -> Self {
        Self::for_sampling_semantics(page.sampling_semantics)
    }

    pub(crate) fn for_sampling_semantics(sampling_semantics: GlyphAtlasSamplingSemantics) -> Self {
        let (shader_decode, blend_mode) = match sampling_semantics {
            GlyphAtlasSamplingSemantics::AlphaCoverage => (
                GlyphAtlasShaderDecode::AlphaCoverage,
                GlyphAtlasBlendMode::StandardAlpha,
            ),
            GlyphAtlasSamplingSemantics::SubpixelCoverage => (
                GlyphAtlasShaderDecode::SubpixelRgbCoverage,
                GlyphAtlasBlendMode::SubpixelBackgroundComposite,
            ),
            GlyphAtlasSamplingSemantics::SignedDistance => (
                GlyphAtlasShaderDecode::SignedDistanceCoverage,
                GlyphAtlasBlendMode::StandardAlpha,
            ),
            GlyphAtlasSamplingSemantics::MultiChannelSignedDistance => (
                GlyphAtlasShaderDecode::MultiChannelSignedDistanceCoverage,
                GlyphAtlasBlendMode::StandardAlpha,
            ),
            GlyphAtlasSamplingSemantics::ColorRgba => (
                GlyphAtlasShaderDecode::ColorRgba,
                GlyphAtlasBlendMode::SourceRgba,
            ),
        };

        Self {
            sampling_semantics,
            shader_decode,
            blend_mode,
        }
    }

    pub(crate) fn requires_background_composite(self) -> bool {
        matches!(
            self.blend_mode,
            GlyphAtlasBlendMode::SubpixelBackgroundComposite
        )
    }

    pub(crate) fn shader_entry_points(self) -> GlyphAtlasShaderEntryPoints {
        GlyphAtlasShaderEntryPoints {
            vertex: GLYPH_ATLAS_VERTEX_ENTRY_POINT,
            fragment: self.shader_decode.fragment_entry_point(),
        }
    }
}

#[cfg(test)]
#[path = "render_contract/tests/cases.rs"]
mod tests;
