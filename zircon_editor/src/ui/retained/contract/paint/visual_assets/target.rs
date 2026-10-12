const MAX_VECTOR_RASTER_EDGE_VALUE: u32 = 4096;
const VECTOR_SMALL_ICON_SUPERSAMPLE_SCALE: u32 = 4;
const VECTOR_SUPERSAMPLE_SCALE: u32 = 2;
const VECTOR_SMALL_ICON_SUPERSAMPLE_MAX_EDGE: u32 = 32;
const VECTOR_RASTER_CACHE_SMALL_EDGE: u32 = 32;
const VECTOR_RASTER_CACHE_MEDIUM_EDGE: u32 = 64;
const VECTOR_RASTER_CACHE_LARGE_EDGE: u32 = 256;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MAX_VECTOR_RASTER_EDGE: u32 =
    MAX_VECTOR_RASTER_EDGE_VALUE;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_ICON_DEFAULT_EDGE: u32 =
    24;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct RasterTargetSize {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) width: u32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) height: u32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn raster_size_from_frame(
    width: f32,
    height: f32,
) -> Option<(u32, u32)> {
    let target = RasterTargetSize::from_frame(width, height)?;
    Some((target.width, target.height))
}

impl RasterTargetSize {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn new(
        width: u32,
        height: u32,
    ) -> Option<Self> {
        (width > 0 && height > 0).then_some(Self { width, height })
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn from_frame(
        width: f32,
        height: f32,
    ) -> Option<Self> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return None;
        }
        Self::new(
            width.ceil().clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32,
            height.ceil().clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32,
        )
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn vector_supersampled_source(
        self,
    ) -> (Self, u32) {
        let supersample_scale = self.vector_supersample_scale();
        let width = self.width.checked_mul(supersample_scale);
        let height = self.height.checked_mul(supersample_scale);
        match (width, height) {
            (Some(width), Some(height))
                if width <= MAX_VECTOR_RASTER_EDGE && height <= MAX_VECTOR_RASTER_EDGE =>
            {
                (Self { width, height }, supersample_scale)
            }
            _ => (self, 1),
        }
    }

    fn vector_supersample_scale(self) -> u32 {
        if self.width.max(self.height) <= VECTOR_SMALL_ICON_SUPERSAMPLE_MAX_EDGE {
            VECTOR_SMALL_ICON_SUPERSAMPLE_SCALE
        } else {
            VECTOR_SUPERSAMPLE_SCALE
        }
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn vector_cache_bucket(
        self,
    ) -> Self {
        // Independent edge quantization changes the aspect ratio and the cached bitmap is later
        // stretched directly into the requested frame. Keep non-square vector targets exact.
        if self.width != self.height {
            return self;
        }
        let max_edge = self.width.max(self.height);
        let bucket_edge = if max_edge <= VECTOR_RASTER_CACHE_SMALL_EDGE {
            1
        } else if max_edge <= VECTOR_RASTER_CACHE_MEDIUM_EDGE {
            4
        } else if max_edge <= VECTOR_RASTER_CACHE_LARGE_EDGE {
            8
        } else {
            16
        };
        self.quantized_up(bucket_edge)
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn fit_preserving_aspect(
        self,
        source_width: f32,
        source_height: f32,
    ) -> Option<Self> {
        if !source_width.is_finite()
            || !source_height.is_finite()
            || source_width <= 0.0
            || source_height <= 0.0
        {
            return None;
        }
        let scale = (self.width as f32 / source_width).min(self.height as f32 / source_height);
        Self::new(
            (source_width * scale).round().clamp(1.0, self.width as f32) as u32,
            (source_height * scale)
                .round()
                .clamp(1.0, self.height as f32) as u32,
        )
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn quantized_up(
        self,
        bucket_edge: u32,
    ) -> Self {
        if bucket_edge <= 1 {
            return self;
        }
        let quantize = |edge: u32| {
            edge.saturating_add(bucket_edge - 1)
                .checked_div(bucket_edge)
                .unwrap_or(edge)
                .saturating_mul(bucket_edge)
                .min(MAX_VECTOR_RASTER_EDGE)
        };
        Self {
            width: quantize(self.width),
            height: quantize(self.height),
        }
    }
}

#[cfg(test)]
#[path = "tests/target.rs"]
mod tests;
