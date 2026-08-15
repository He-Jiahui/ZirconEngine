const MAX_VECTOR_RASTER_EDGE_VALUE: u32 = 4096;
const VECTOR_SUPERSAMPLE_SCALE: u32 = 2;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MAX_VECTOR_RASTER_EDGE: u32 =
    MAX_VECTOR_RASTER_EDGE_VALUE;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUI_ICON_DEFAULT_EDGE: u32 =
    24;

#[derive(Clone, Copy)]
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
        let width = self.width.checked_mul(VECTOR_SUPERSAMPLE_SCALE);
        let height = self.height.checked_mul(VECTOR_SUPERSAMPLE_SCALE);
        match (width, height) {
            (Some(width), Some(height))
                if width <= MAX_VECTOR_RASTER_EDGE && height <= MAX_VECTOR_RASTER_EDGE =>
            {
                (Self { width, height }, VECTOR_SUPERSAMPLE_SCALE)
            }
            _ => (self, 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RasterTargetSize, MAX_VECTOR_RASTER_EDGE};

    #[test]
    fn vector_source_uses_bounded_two_x_local_supersampling() {
        let target = RasterTargetSize::new(20, 18).expect("display target");
        let (source, scale) = target.vector_supersampled_source();

        assert_eq!(scale, 2);
        assert_eq!((source.width, source.height), (40, 36));
    }

    #[test]
    fn vector_source_never_exceeds_the_raster_edge_limit() {
        let target = RasterTargetSize::new(MAX_VECTOR_RASTER_EDGE, 32).expect("large target");
        let (source, scale) = target.vector_supersampled_source();

        assert_eq!(scale, 1);
        assert_eq!(source.width, MAX_VECTOR_RASTER_EDGE);
        assert_eq!(source.height, 32);
    }
}
