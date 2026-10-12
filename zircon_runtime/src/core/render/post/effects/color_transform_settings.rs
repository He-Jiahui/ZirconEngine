use crate::core::math::Real;
use crate::core::resource::{ResourceHandle, TextureMarker};

use crate::core::framework::render::{RenderImageDescriptor, RenderImageDimension};

pub const MIN_COLOR_LOOKUP_TEXTURE_SIZE: u32 = 2;
pub const MAX_COLOR_LOOKUP_TEXTURE_SIZE: u32 = 256;
const MIN_TONEMAP_WHITE_POINT: Real = 0.001;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderTonemapOperator {
    #[default]
    None,
    Reinhard,
    Aces,
    Filmic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderTonemapSettings {
    pub operator: RenderTonemapOperator,
    pub exposure_bias: Real,
    pub white_point: Real,
}

impl Default for RenderTonemapSettings {
    fn default() -> Self {
        Self {
            operator: RenderTonemapOperator::None,
            exposure_bias: 0.0,
            white_point: 1.0,
        }
    }
}

impl RenderTonemapSettings {
    pub fn is_enabled(self) -> bool {
        self.operator != RenderTonemapOperator::None
            || self.exposure_bias != 0.0
            || self.white_point != 1.0
    }

    pub fn render_operator_id(self) -> u32 {
        match self.operator {
            RenderTonemapOperator::None => 0,
            RenderTonemapOperator::Reinhard => 1,
            RenderTonemapOperator::Aces => 2,
            RenderTonemapOperator::Filmic => 3,
        }
    }

    pub fn render_exposure_bias(self) -> Real {
        self.exposure_bias
    }

    pub fn render_white_point(self) -> Real {
        self.white_point.max(MIN_TONEMAP_WHITE_POINT)
    }
}

/// LUT 布局声明要求尺寸与纹理维度严格匹配：二维条带为 size²×size，三维纹理为 size³；Auto 只按现有描述符推断，不申请资源。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderColorLookupTextureLayout {
    #[default]
    Auto,
    Texture2dStrip {
        size: u32,
    },
    Texture3d {
        size: u32,
    },
}

impl RenderColorLookupTextureLayout {
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Texture2dStrip { .. } => "texture-2d-strip",
            Self::Texture3d { .. } => "texture-3d",
        }
    }

    pub fn requested_size(self) -> Option<u32> {
        match self {
            Self::Auto => None,
            Self::Texture2dStrip { size } | Self::Texture3d { size } => Some(size),
        }
    }

    pub fn has_valid_requested_size(self) -> bool {
        self.requested_size().is_none_or(|size| {
            (MIN_COLOR_LOOKUP_TEXTURE_SIZE..=MAX_COLOR_LOOKUP_TEXTURE_SIZE).contains(&size)
        })
    }

    pub fn matches_texture_2d_strip(self, descriptor: &RenderImageDescriptor) -> bool {
        let size = match self {
            Self::Auto => {
                let Some(size) = inferred_strip_size(descriptor) else {
                    return false;
                };
                size
            }
            Self::Texture2dStrip { size } => size,
            Self::Texture3d { .. } => return false,
        };
        self.has_valid_requested_size()
            && descriptor.dimension == RenderImageDimension::D2
            && descriptor.width == size.saturating_mul(size)
            && descriptor.height == size
            && descriptor.depth_or_array_layers == 1
    }

    pub fn matches_texture_3d(self, descriptor: &RenderImageDescriptor) -> bool {
        let size = match self {
            Self::Auto => return descriptor.dimension == RenderImageDimension::D3,
            Self::Texture3d { size } => size,
            Self::Texture2dStrip { .. } => return false,
        };
        self.has_valid_requested_size()
            && descriptor.dimension == RenderImageDimension::D3
            && descriptor.width == size
            && descriptor.height == size
            && descriptor.depth_or_array_layers == size
    }

    pub fn accepts_current_post_process_binding(self, descriptor: &RenderImageDescriptor) -> bool {
        match self {
            Self::Auto => {
                descriptor.dimension == RenderImageDimension::D2
                    && descriptor.depth_or_array_layers == 1
            }
            Self::Texture2dStrip { .. } => self.matches_texture_2d_strip(descriptor),
            Self::Texture3d { .. } => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderColorLookupSettings {
    pub texture: Option<ResourceHandle<TextureMarker>>,
    pub texture_layout: RenderColorLookupTextureLayout,
    pub intensity: Real,
}

impl Default for RenderColorLookupSettings {
    fn default() -> Self {
        Self {
            texture: None,
            texture_layout: RenderColorLookupTextureLayout::Auto,
            intensity: 0.0,
        }
    }
}

impl RenderColorLookupSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0
    }

    pub fn render_intensity(self) -> Real {
        self.intensity.max(0.0)
    }
}

fn inferred_strip_size(descriptor: &RenderImageDescriptor) -> Option<u32> {
    let height = descriptor.height;
    (height > 0 && descriptor.width == height.saturating_mul(height)).then_some(height)
}

#[cfg(test)]
#[path = "tests/color_transform_settings.rs"]
mod tests;
