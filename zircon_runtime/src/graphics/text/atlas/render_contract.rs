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
mod tests;
