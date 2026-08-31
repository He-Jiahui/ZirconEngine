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
mod tests {
    use std::collections::BTreeSet;

    use super::{RasterTargetSize, MAX_VECTOR_RASTER_EDGE};

    #[test]
    fn small_icon_vector_source_uses_four_x_local_supersampling() {
        let target = RasterTargetSize::new(20, 18).expect("display target");
        let (source, scale) = target.vector_supersampled_source();

        assert_eq!(scale, 4);
        assert_eq!((source.width, source.height), (80, 72));
    }

    #[test]
    fn larger_vector_source_uses_two_x_local_supersampling() {
        let target = RasterTargetSize::new(64, 48).expect("display target");
        let (source, scale) = target.vector_supersampled_source();

        assert_eq!(scale, 2);
        assert_eq!((source.width, source.height), (128, 96));
    }

    #[test]
    fn vector_source_never_exceeds_the_raster_edge_limit() {
        let target = RasterTargetSize::new(MAX_VECTOR_RASTER_EDGE, 32).expect("large target");
        let (source, scale) = target.vector_supersampled_source();

        assert_eq!(scale, 1);
        assert_eq!(source.width, MAX_VECTOR_RASTER_EDGE);
        assert_eq!(source.height, 32);
    }

    #[test]
    fn raster_target_fits_non_square_sources_without_distortion() {
        let target = RasterTargetSize::new(100, 100).expect("display target");

        let wide = target
            .fit_preserving_aspect(200.0, 100.0)
            .expect("wide source");
        let tall = target
            .fit_preserving_aspect(100.0, 200.0)
            .expect("tall source");

        assert_eq!((wide.width, wide.height), (100, 50));
        assert_eq!((tall.width, tall.height), (50, 100));
    }

    #[test]
    fn preview_raster_targets_quantize_up_without_exceeding_the_edge_limit() {
        let target = RasterTargetSize::new(121, 125).expect("physical target");
        let bounded =
            RasterTargetSize::new(MAX_VECTOR_RASTER_EDGE - 1, 1).expect("bounded physical target");

        assert_eq!(
            target.quantized_up(8),
            RasterTargetSize::new(128, 128).unwrap()
        );
        assert_eq!(
            bounded.quantized_up(8),
            RasterTargetSize::new(MAX_VECTOR_RASTER_EDGE, 8).unwrap()
        );
        assert_eq!(target.quantized_up(1), target);
    }

    #[test]
    fn continuous_resize_uses_bounded_vector_cache_buckets() {
        let exact_sizes = (1..=512)
            .map(|edge| RasterTargetSize::new(edge, edge).unwrap())
            .collect::<BTreeSet<_>>();
        let bucketed_sizes = exact_sizes
            .iter()
            .copied()
            .map(RasterTargetSize::vector_cache_bucket)
            .collect::<BTreeSet<_>>();

        assert_eq!(exact_sizes.len(), 512);
        assert_eq!(bucketed_sizes.len(), 80);
        assert_eq!(
            RasterTargetSize::new(17, 19).unwrap().vector_cache_bucket(),
            RasterTargetSize::new(17, 19).unwrap()
        );
        assert_eq!(
            RasterTargetSize::new(121, 125)
                .unwrap()
                .vector_cache_bucket(),
            RasterTargetSize::new(121, 125).unwrap()
        );
    }

    #[test]
    fn non_square_vector_targets_keep_their_physical_aspect_ratio() {
        let target = RasterTargetSize::new(41, 43).expect("non-square physical target");

        assert_eq!(target.vector_cache_bucket(), target);
    }

    #[test]
    fn frame_target_ceil_preserves_fractional_physical_pixel_coverage() {
        let at_125_percent =
            RasterTargetSize::from_frame(17.0 * 1.25, 13.0 * 1.25).expect("125% target");
        let at_150_percent =
            RasterTargetSize::from_frame(17.0 * 1.5, 13.0 * 1.5).expect("150% target");

        assert_eq!((at_125_percent.width, at_125_percent.height), (22, 17));
        assert_eq!((at_150_percent.width, at_150_percent.height), (26, 20));
    }

    #[test]
    fn frame_target_clamps_after_rounding_up_to_the_physical_edge_limit() {
        let target = RasterTargetSize::from_frame(
            MAX_VECTOR_RASTER_EDGE as f32 + 0.25,
            MAX_VECTOR_RASTER_EDGE as f32 + 128.0,
        )
        .expect("bounded target");

        assert_eq!(target.width, MAX_VECTOR_RASTER_EDGE);
        assert_eq!(target.height, MAX_VECTOR_RASTER_EDGE);
    }
}
