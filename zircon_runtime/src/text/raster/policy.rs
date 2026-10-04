//! 按格式、缩放需求和效果选择位图或距离场路径，供 UI 自动模式与距离场效果准备共享。
//! 自动模式在大小阈值附近保留已热身的路径，减少尺寸轻微变化导致缓存和图集来回切换。

use crate::text::atlas::GlyphAtlasFormat;
use crate::text::sdf::SdfMode;

const DEFAULT_SDF_MIN_SIZE_PX: f32 = 24.0;
const DEFAULT_SDF_HYSTERESIS_PX: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphRasterPath {
    Bitmap,
    Sdf,
    Msdf,
    Mtsdf,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphRasterEffects {
    pub(crate) outline: bool,
    pub(crate) shadow: bool,
    pub(crate) glow: bool,
    pub(crate) true_distance_effects: bool,
}

impl GlyphRasterEffects {
    fn requires_distance_field(self) -> bool {
        self.outline || self.shadow || self.glow
    }

    fn requires_true_distance(self) -> bool {
        self.requires_distance_field() && self.true_distance_effects
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// 由调用方提供已解析的字号与效果需求；这是路由信息，不验证字体或生成像素。
/// 颜色和子像素格式优先留在位图路径；轮廓/阴影/发光需要距离场，真实距离效果要求 MTSDF。
pub(crate) struct GlyphRasterPolicyRequest {
    pub(crate) size_px: f32,
    pub(crate) scalable: bool,
    pub(crate) requested_format: GlyphAtlasFormat,
    pub(crate) effects: GlyphRasterEffects,
}

impl GlyphRasterPolicyRequest {
    pub(crate) fn new(size_px: f32, scalable: bool) -> Self {
        Self {
            size_px,
            scalable,
            requested_format: GlyphAtlasFormat::AlphaMask,
            effects: GlyphRasterEffects::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphRasterPolicy {
    pub(crate) sdf_min_size_px: f32,
    pub(crate) scalable_prefers_sdf: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphRasterAutoPolicyDecision {
    pub(crate) path: GlyphRasterPath,
    pub(crate) retained_warm_path: bool,
}

impl Default for GlyphRasterPolicy {
    fn default() -> Self {
        Self {
            sdf_min_size_px: DEFAULT_SDF_MIN_SIZE_PX,
            scalable_prefers_sdf: true,
        }
    }
}

pub(crate) fn raster_path_for(size_px: f32, scalable: bool) -> GlyphRasterPath {
    GlyphRasterPolicy::default().path_for(size_px, scalable)
}

pub(crate) fn raster_path_for_request(request: GlyphRasterPolicyRequest) -> GlyphRasterPath {
    GlyphRasterPolicy::default().path_for_request(request)
}

pub(crate) fn auto_raster_path_for_request(
    request: GlyphRasterPolicyRequest,
    warm_path: Option<GlyphRasterPath>,
) -> GlyphRasterAutoPolicyDecision {
    GlyphRasterPolicy::default().auto_path_for_request(request, warm_path)
}

pub(crate) fn distance_field_mode_for_request(
    request: GlyphRasterPolicyRequest,
) -> Option<SdfMode> {
    match raster_path_for_request(request) {
        GlyphRasterPath::Bitmap => None,
        GlyphRasterPath::Sdf => Some(SdfMode::Sdf),
        GlyphRasterPath::Msdf => Some(SdfMode::Msdf),
        GlyphRasterPath::Mtsdf => Some(SdfMode::Mtsdf),
    }
}

impl GlyphRasterPolicy {
    pub(crate) fn path_for(self, size_px: f32, scalable: bool) -> GlyphRasterPath {
        self.path_for_request(GlyphRasterPolicyRequest::new(size_px, scalable))
    }

    pub(crate) fn path_for_request(self, request: GlyphRasterPolicyRequest) -> GlyphRasterPath {
        match request.requested_format {
            GlyphAtlasFormat::SubpixelMask | GlyphAtlasFormat::Color => {
                return GlyphRasterPath::Bitmap;
            }
            _ if request.effects.requires_true_distance() => return GlyphRasterPath::Mtsdf,
            GlyphAtlasFormat::Sdf => return GlyphRasterPath::Sdf,
            GlyphAtlasFormat::Msdf => return GlyphRasterPath::Msdf,
            GlyphAtlasFormat::AlphaMask => {}
        }

        if request.effects.requires_distance_field() {
            return GlyphRasterPath::Sdf;
        }

        if request.scalable && self.scalable_prefers_sdf {
            return GlyphRasterPath::Sdf;
        }

        if request.size_px >= self.sdf_min_size_px {
            GlyphRasterPath::Sdf
        } else {
            GlyphRasterPath::Bitmap
        }
    }

    // 用于普通静态 alpha 文字的自动路由，warm_path 应来自同一文字身份的上次成功路径。
    // 显式格式、缩放文字或距离场效果不延续旧位图决定，避免迟滞覆盖调用方的强制要求。
    pub(crate) fn auto_path_for_request(
        self,
        request: GlyphRasterPolicyRequest,
        warm_path: Option<GlyphRasterPath>,
    ) -> GlyphRasterAutoPolicyDecision {
        let cold_path = self.path_for_request(request);
        let plain_scalable_alpha = matches!(request.requested_format, GlyphAtlasFormat::AlphaMask)
            && !request.scalable
            && !request.effects.requires_distance_field();
        if !plain_scalable_alpha {
            return GlyphRasterAutoPolicyDecision {
                path: cold_path,
                retained_warm_path: false,
            };
        }

        let lower_bound = (self.sdf_min_size_px - DEFAULT_SDF_HYSTERESIS_PX).max(0.0);
        let upper_bound = self.sdf_min_size_px + DEFAULT_SDF_HYSTERESIS_PX;
        let retained_path = match warm_path {
            Some(GlyphRasterPath::Bitmap) if request.size_px < upper_bound => {
                Some(GlyphRasterPath::Bitmap)
            }
            Some(GlyphRasterPath::Sdf) if request.size_px >= lower_bound => {
                Some(GlyphRasterPath::Sdf)
            }
            _ => None,
        };

        GlyphRasterAutoPolicyDecision {
            path: retained_path.unwrap_or(cold_path),
            retained_warm_path: retained_path.is_some()
                && request.size_px >= lower_bound
                && request.size_px < upper_bound,
        }
    }
}

#[cfg(test)]
#[path = "tests/policy.rs"]
mod tests;
