#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UiGlyphRasterPath {
    Sdf,
    Bitmap,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiGlyphRasterPolicy {
    pub sdf_min_size_px: f32,
    pub scalable_prefers_sdf: bool,
}

impl Default for UiGlyphRasterPolicy {
    fn default() -> Self {
        Self {
            sdf_min_size_px: 24.0,
            scalable_prefers_sdf: true,
        }
    }
}

pub(crate) fn raster_path_for(size_px: f32, scalable: bool) -> UiGlyphRasterPath {
    UiGlyphRasterPolicy::default().path_for(size_px, scalable)
}

impl UiGlyphRasterPolicy {
    pub(crate) fn path_for(self, size_px: f32, scalable: bool) -> UiGlyphRasterPath {
        if scalable && self.scalable_prefers_sdf {
            return UiGlyphRasterPath::Sdf;
        }

        if size_px >= self.sdf_min_size_px {
            UiGlyphRasterPath::Sdf
        } else {
            UiGlyphRasterPath::Bitmap
        }
    }
}
