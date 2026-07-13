use crate::graphics::text::atlas::GlyphAtlasFormat;

/// Pixel encoding chosen after shaping and before glyph atlas allocation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SdfMode {
    #[default]
    Sdf,
    Msdf,
    Mtsdf,
}

impl SdfMode {
    pub(crate) const SDF_SHADER_DISCRIMINANT: u32 = 0;
    pub(crate) const MSDF_SHADER_DISCRIMINANT: u32 = 1;
    pub(crate) const MTSDF_SHADER_DISCRIMINANT: u32 = 2;

    pub(crate) const fn channel_count(self) -> u8 {
        match self {
            Self::Sdf => 1,
            Self::Msdf | Self::Mtsdf => 4,
        }
    }

    pub(crate) const fn atlas_format(self) -> GlyphAtlasFormat {
        match self {
            Self::Sdf => GlyphAtlasFormat::Sdf,
            Self::Msdf | Self::Mtsdf => GlyphAtlasFormat::Msdf,
        }
    }

    pub(crate) const fn shader_discriminant(self) -> u32 {
        match self {
            Self::Sdf => Self::SDF_SHADER_DISCRIMINANT,
            Self::Msdf => Self::MSDF_SHADER_DISCRIMINANT,
            Self::Mtsdf => Self::MTSDF_SHADER_DISCRIMINANT,
        }
    }

    pub(crate) const fn from_shader_discriminant(value: u32) -> Option<Self> {
        match value {
            Self::SDF_SHADER_DISCRIMINANT => Some(Self::Sdf),
            Self::MSDF_SHADER_DISCRIMINANT => Some(Self::Msdf),
            Self::MTSDF_SHADER_DISCRIMINANT => Some(Self::Mtsdf),
            _ => None,
        }
    }
}
