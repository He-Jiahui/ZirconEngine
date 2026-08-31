use crate::text::InstancedFaceId;

use super::GlyphAtlasFormat;

const DEFAULT_PX_SIZE_BUCKET_QUANTUM: f32 = 1.0;
const SUBPIXEL_BIN_COUNT: u8 = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GlyphHintingMode {
    #[default]
    None,
    Vertical,
    Full,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GlyphSmoothingMode {
    None,
    #[default]
    Grayscale,
    Subpixel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SyntheticGlyphStyle {
    pub(crate) bold: bool,
    pub(crate) oblique: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphRasterRequest {
    pub(crate) face: InstancedFaceId,
    pub(crate) glyph_id: u32,
    pub(crate) logical_px: f32,
    pub(crate) scale_factor: f32,
    pub(crate) screen_x: f32,
    pub(crate) snap_to_pixel: bool,
    pub(crate) format: GlyphAtlasFormat,
    pub(crate) hinting: GlyphHintingMode,
    pub(crate) smoothing: GlyphSmoothingMode,
    pub(crate) synthetic: SyntheticGlyphStyle,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GlyphRasterPlacement {
    pub(crate) requested_x: f32,
    pub(crate) snapped_x: f32,
    pub(crate) subpixel_bin: u8,
}

impl GlyphRasterPlacement {
    pub(crate) fn from_request(request: GlyphRasterRequest) -> Self {
        Self::from_raster_input(
            request.format,
            request.smoothing,
            request.snap_to_pixel,
            request.screen_x,
        )
    }

    pub(crate) fn from_raster_input(
        format: GlyphAtlasFormat,
        smoothing: GlyphSmoothingMode,
        snap_to_pixel: bool,
        screen_x: f32,
    ) -> Self {
        let requested_x = finite_or_default(screen_x, 0.0);
        if snap_to_pixel {
            return Self {
                requested_x,
                snapped_x: requested_x.round(),
                subpixel_bin: 0,
            };
        }

        if smoothing != GlyphSmoothingMode::Subpixel || !format.supports_subpixel_bins() {
            return Self {
                requested_x,
                snapped_x: requested_x,
                subpixel_bin: 0,
            };
        }

        let subpixel_bin = subpixel_bin_for_screen_x(requested_x);
        // Render extract must use the same bin origin that selected the raster bitmap.
        let snapped_x = requested_x.floor() + subpixel_bin as f32 / SUBPIXEL_BIN_COUNT as f32;
        Self {
            requested_x,
            snapped_x,
            subpixel_bin,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct GlyphRasterKey {
    pub(crate) face: InstancedFaceId,
    pub(crate) glyph_id: u32,
    pub(crate) px_size_bucket: u32,
    pub(crate) subpixel_bin: u8,
    /// Vertical phase is retained for backends whose cache key rasterizes y subpixel offsets.
    pub(crate) vertical_subpixel_bin: u8,
    pub(crate) format: GlyphAtlasFormat,
    pub(crate) hinting: GlyphHintingMode,
    pub(crate) smoothing: GlyphSmoothingMode,
    pub(crate) synthetic: SyntheticGlyphStyle,
}

impl GlyphRasterKey {
    pub(crate) fn from_request(request: GlyphRasterRequest) -> Self {
        Self::from_request_with_quantum(request, DEFAULT_PX_SIZE_BUCKET_QUANTUM)
    }

    pub(crate) fn from_request_with_quantum(request: GlyphRasterRequest, quantum_px: f32) -> Self {
        let px_size_bucket = px_size_bucket(request.logical_px, request.scale_factor, quantum_px);
        let subpixel_bin = GlyphRasterPlacement::from_request(request).subpixel_bin;
        let (hinting, smoothing) =
            normalized_raster_quality(request.format, request.hinting, request.smoothing);

        Self {
            face: request.face,
            glyph_id: request.glyph_id,
            px_size_bucket,
            subpixel_bin,
            vertical_subpixel_bin: 0,
            format: request.format,
            hinting,
            smoothing,
            synthetic: request.synthetic,
        }
    }
}

fn px_size_bucket(logical_px: f32, scale_factor: f32, quantum_px: f32) -> u32 {
    let quantum_px = finite_positive_or_default(quantum_px, DEFAULT_PX_SIZE_BUCKET_QUANTUM);
    let physical_px =
        finite_positive_or_default(logical_px, 1.0) * finite_positive_or_default(scale_factor, 1.0);
    ((physical_px / quantum_px).round() * quantum_px)
        .round()
        .max(1.0) as u32
}

fn subpixel_bin_for_screen_x(screen_x: f32) -> u8 {
    let fraction = positive_fraction(screen_x);
    (fraction * SUBPIXEL_BIN_COUNT as f32).floor() as u8
}

fn positive_fraction(value: f32) -> f32 {
    let fraction = value.fract();
    if fraction < 0.0 {
        fraction + 1.0
    } else {
        fraction
    }
}

fn normalized_raster_quality(
    format: GlyphAtlasFormat,
    hinting: GlyphHintingMode,
    smoothing: GlyphSmoothingMode,
) -> (GlyphHintingMode, GlyphSmoothingMode) {
    if format.is_distance_field() {
        (GlyphHintingMode::None, GlyphSmoothingMode::None)
    } else {
        (hinting, smoothing)
    }
}

fn finite_or_default(value: f32, default_value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        default_value
    }
}

fn finite_positive_or_default(value: f32, default_value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        default_value
    }
}

impl GlyphAtlasFormat {
    fn is_distance_field(self) -> bool {
        matches!(self, Self::Sdf | Self::Msdf)
    }

    fn supports_subpixel_bins(self) -> bool {
        matches!(self, Self::AlphaMask | Self::SubpixelMask)
    }
}

#[cfg(test)]
mod tests;
