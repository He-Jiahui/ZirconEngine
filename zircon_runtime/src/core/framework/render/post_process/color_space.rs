use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderPostProcessTextureFormat {
    R8Unorm,
    Rg16Float,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rg11b10Ufloat,
    Rgba16Float,
    Rgba32Float,
}

impl RenderPostProcessTextureFormat {
    pub const fn label(self) -> &'static str {
        match self {
            Self::R8Unorm => "r8unorm",
            Self::Rg16Float => "rg16float",
            Self::Rgba8Unorm => "rgba8unorm",
            Self::Rgba8UnormSrgb => "rgba8unorm-srgb",
            Self::Rg11b10Ufloat => "rg11b10ufloat",
            Self::Rgba16Float => "rgba16float",
            Self::Rgba32Float => "rgba32float",
        }
    }

    pub const fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::R8Unorm => 1,
            Self::Rg16Float | Self::Rgba8Unorm | Self::Rgba8UnormSrgb | Self::Rg11b10Ufloat => 4,
            Self::Rgba16Float => 8,
            Self::Rgba32Float => 16,
        }
    }

    pub const fn is_hdr_color(self) -> bool {
        matches!(
            self,
            Self::Rg16Float | Self::Rg11b10Ufloat | Self::Rgba16Float | Self::Rgba32Float
        )
    }
}

impl fmt::Display for RenderPostProcessTextureFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenderOutputTransfer {
    #[default]
    SrgbNonlinear,
    LinearExtended,
    Hdr10Pq,
}

impl RenderOutputTransfer {
    pub const fn label(self) -> &'static str {
        match self {
            Self::SrgbNonlinear => "srgb-nonlinear",
            Self::LinearExtended => "linear-extended",
            Self::Hdr10Pq => "hdr10-pq",
        }
    }
}

impl fmt::Display for RenderOutputTransfer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

pub const INTERMEDIATE_HDR_FORMAT_DEFAULT: RenderPostProcessTextureFormat =
    RenderPostProcessTextureFormat::Rg11b10Ufloat;
pub const INTERMEDIATE_HDR_FORMAT_HIGH_QUALITY: RenderPostProcessTextureFormat =
    RenderPostProcessTextureFormat::Rgba16Float;
pub const COLOR_LUT_SIZE_DEFAULT: u32 = 32;
pub const COLOR_LUT_SIZE_HIGH_QUALITY: u32 = 64;
pub const COLOR_LUT_FORMAT: RenderPostProcessTextureFormat =
    RenderPostProcessTextureFormat::Rgba16Float;
pub const TONEMAPPED_SDR_FORMAT: RenderPostProcessTextureFormat =
    RenderPostProcessTextureFormat::Rgba8Unorm;
pub const OUTPUT_TRANSFER_DEFAULT: RenderOutputTransfer = RenderOutputTransfer::SrgbNonlinear;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_post_color_space_intermediate_hdr_defaults_to_rg11b10ufloat() {
        assert_eq!(
            INTERMEDIATE_HDR_FORMAT_DEFAULT,
            RenderPostProcessTextureFormat::Rg11b10Ufloat
        );
        assert_eq!(INTERMEDIATE_HDR_FORMAT_DEFAULT.bytes_per_pixel(), 4);
        assert!(INTERMEDIATE_HDR_FORMAT_DEFAULT.is_hdr_color());
    }

    #[test]
    fn render_post_color_lut_contract_keeps_power_of_two_sizes() {
        assert_eq!(COLOR_LUT_SIZE_DEFAULT, 32);
        assert_eq!(COLOR_LUT_SIZE_HIGH_QUALITY, 64);
        assert_eq!(
            COLOR_LUT_FORMAT,
            RenderPostProcessTextureFormat::Rgba16Float
        );
    }

    #[test]
    fn render_post_output_transfer_defaults_to_srgb() {
        assert_eq!(OUTPUT_TRANSFER_DEFAULT, RenderOutputTransfer::SrgbNonlinear);
        assert_eq!(OUTPUT_TRANSFER_DEFAULT.label(), "srgb-nonlinear");
    }
}
