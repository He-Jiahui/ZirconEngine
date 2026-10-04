use std::fmt;

/// 后处理纹理的中立格式契约，用于选择中间纹理及 LUT 布局；最终输出传输由 RenderOutputTransfer 另行声明。
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

// The baseline device requests no optional format features. Packed R11G11B10 needs an
// explicit render-target capability, including on adapters that support the format.
pub const INTERMEDIATE_HDR_FORMAT_DEFAULT: RenderPostProcessTextureFormat =
    RenderPostProcessTextureFormat::Rgba16Float;
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
#[path = "tests/color_space.rs"]
mod tests;
