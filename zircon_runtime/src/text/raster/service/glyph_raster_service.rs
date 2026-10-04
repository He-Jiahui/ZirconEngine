use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::core::framework::text::{
    TextGlyphBitmapFormat, TextGlyphRasterError, TextGlyphRasterMode, TextGlyphRasterReceipt,
    TextGlyphRasterRequest, TextGlyphRasterSmoothing,
};
use crate::text::VariationCoords;

use super::RuntimeGlyphRasterFace;
use crate::text::raster::swash::{
    GlyphBitmapContent, SwashRasterError, SwashRasterRequest, SwashRasterizer,
};

const GLYPH_RASTER_PROFILE_COUNTER_NAMES: [&str; 16] = [
    "text_glyph_raster_request_count",
    "text_glyph_raster_outline_request_count",
    "text_glyph_raster_color_preferred_request_count",
    "text_glyph_raster_subpixel_request_count",
    "text_glyph_raster_lock_wait_nanos",
    "text_glyph_raster_lock_hold_nanos",
    "text_glyph_raster_backend_nanos",
    "text_glyph_raster_success_count",
    "text_glyph_raster_failure_count",
    "text_glyph_raster_bitmap_bytes",
    "text_glyph_raster_alpha_bitmap_count",
    "text_glyph_raster_subpixel_bitmap_count",
    "text_glyph_raster_color_bitmap_count",
    "text_glyph_raster_alpha_backend_nanos",
    "text_glyph_raster_subpixel_backend_nanos",
    "text_glyph_raster_color_backend_nanos",
];

pub(crate) struct GlyphRasterService {
    rasterizer: Mutex<SwashRasterizer>,
}

impl GlyphRasterService {
    pub(crate) fn new() -> Self {
        Self {
            rasterizer: Mutex::new(SwashRasterizer::new()),
        }
    }

    pub(crate) fn rasterize(
        &self,
        face: RuntimeGlyphRasterFace<'_>,
        request: TextGlyphRasterRequest,
    ) -> Result<TextGlyphRasterReceipt, TextGlyphRasterError> {
        crate::profile_scope!("runtime", "text.glyph_raster", "rasterize");
        record_request(request);
        let raster_request =
            match SwashRasterRequest::from_text_glyph_request(face.collection_index, request) {
                Ok(request) => request,
                Err(error) => {
                    record_failure();
                    return Err(error);
                }
            }
            .with_font_identity(font_identity(face.source_identity))
            .with_variations(Arc::new(
                face.variations
                    .cloned()
                    .unwrap_or_else(VariationCoords::default),
            ));

        let lock_started = profile_started();
        let mut rasterizer = self
            .rasterizer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let lock_wait_elapsed = profile_elapsed(lock_started);
        let lock_hold_started = profile_started();
        let backend_started = profile_started();
        let raster_result = rasterizer.rasterize(face.bytes, raster_request);
        let backend_elapsed = profile_elapsed(backend_started);
        let lock_hold_elapsed = profile_elapsed(lock_hold_started);
        drop(rasterizer);
        record_timing(GLYPH_RASTER_PROFILE_COUNTER_NAMES[4], lock_wait_elapsed);
        record_timing(GLYPH_RASTER_PROFILE_COUNTER_NAMES[5], lock_hold_elapsed);
        record_timing(GLYPH_RASTER_PROFILE_COUNTER_NAMES[6], backend_elapsed);
        let bitmap = match raster_result {
            Ok(bitmap) => bitmap,
            Err(error) => {
                record_failure();
                return Err(map_swash_error(error));
            }
        };
        let format = match bitmap.content {
            GlyphBitmapContent::AlphaMask => TextGlyphBitmapFormat::AlphaMask,
            GlyphBitmapContent::SubpixelMask => TextGlyphBitmapFormat::SubpixelMask,
            GlyphBitmapContent::Color => TextGlyphBitmapFormat::ColorRgba,
        };
        record_success(format, bitmap.data.len(), backend_elapsed);

        Ok(TextGlyphRasterReceipt {
            font_collection: face.font_collection,
            font_face: face.font_face,
            font_instance: face.font_instance,
            font_generation: face.font_generation,
            source_identity: face.source_identity,
            request,
            format,
            size: [bitmap.size.x, bitmap.size.y],
            bearing: [bitmap.bearing.x, bitmap.bearing.y],
            bitmap: Arc::from(bitmap.data),
        })
    }
}

fn record_request(request: TextGlyphRasterRequest) {
    crate::profile_counter!("runtime", GLYPH_RASTER_PROFILE_COUNTER_NAMES[0], 1);
    crate::profile_counter!(
        "runtime",
        GLYPH_RASTER_PROFILE_COUNTER_NAMES[1],
        u8::from(request.mode == TextGlyphRasterMode::Outline)
    );
    crate::profile_counter!(
        "runtime",
        GLYPH_RASTER_PROFILE_COUNTER_NAMES[2],
        u8::from(request.mode == TextGlyphRasterMode::ColorPreferred)
    );
    crate::profile_counter!(
        "runtime",
        GLYPH_RASTER_PROFILE_COUNTER_NAMES[3],
        u8::from(request.smoothing == TextGlyphRasterSmoothing::Subpixel)
    );
}

fn record_success(
    format: TextGlyphBitmapFormat,
    bitmap_bytes: usize,
    backend_elapsed: Option<Duration>,
) {
    crate::profile_counter!("runtime", GLYPH_RASTER_PROFILE_COUNTER_NAMES[7], 1);
    crate::profile_counter!(
        "runtime",
        GLYPH_RASTER_PROFILE_COUNTER_NAMES[9],
        bitmap_bytes
    );
    crate::profile_counter!(
        "runtime",
        GLYPH_RASTER_PROFILE_COUNTER_NAMES[10],
        u8::from(format == TextGlyphBitmapFormat::AlphaMask)
    );
    crate::profile_counter!(
        "runtime",
        GLYPH_RASTER_PROFILE_COUNTER_NAMES[11],
        u8::from(format == TextGlyphBitmapFormat::SubpixelMask)
    );
    crate::profile_counter!(
        "runtime",
        GLYPH_RASTER_PROFILE_COUNTER_NAMES[12],
        u8::from(format == TextGlyphBitmapFormat::ColorRgba)
    );
    let route_counter = match format {
        TextGlyphBitmapFormat::AlphaMask => GLYPH_RASTER_PROFILE_COUNTER_NAMES[13],
        TextGlyphBitmapFormat::SubpixelMask => GLYPH_RASTER_PROFILE_COUNTER_NAMES[14],
        TextGlyphBitmapFormat::ColorRgba => GLYPH_RASTER_PROFILE_COUNTER_NAMES[15],
    };
    record_timing(route_counter, backend_elapsed);
}

fn record_failure() {
    crate::profile_counter!("runtime", GLYPH_RASTER_PROFILE_COUNTER_NAMES[8], 1);
}

fn profile_started() -> Option<Instant> {
    profile_metrics_enabled().then(Instant::now)
}

fn profile_elapsed(started: Option<Instant>) -> Option<Duration> {
    started.map(|started| started.elapsed())
}

fn record_timing(name: &'static str, elapsed: Option<Duration>) {
    let Some(elapsed) = elapsed else {
        return;
    };
    crate::profile_counter!("runtime", name, duration_to_nanos(elapsed));
}

fn duration_to_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

fn profile_metrics_enabled() -> bool {
    #[cfg(feature = "profiling-tracy")]
    {
        return true;
    }
    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    {
        return crate::core::diagnostics::profiling::capture_active();
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    {
        false
    }
}

fn font_identity(source_identity: [u8; 16]) -> [u64; 2] {
    let identity = u128::from_le_bytes(source_identity);
    let lower = identity as u64;
    let upper = (identity >> u64::BITS) as u64;
    [lower, upper]
}

fn map_swash_error(error: SwashRasterError) -> TextGlyphRasterError {
    match error {
        SwashRasterError::InvalidFontFace { .. } => TextGlyphRasterError::MissingFontFace,
        SwashRasterError::MissingGlyphImage { .. } => TextGlyphRasterError::MissingGlyph,
        SwashRasterError::InvalidGlyphBitmap(_) => TextGlyphRasterError::InvalidBitmap,
        SwashRasterError::InvalidPxSize
        | SwashRasterError::InvalidOffset
        | SwashRasterError::InvalidVariationCoordinate => TextGlyphRasterError::BackendUnavailable,
    }
}

#[cfg(test)]
#[path = "tests/glyph_raster_service.rs"]
mod tests;

#[cfg(test)]
#[path = "glyph_raster_service/tests/performance_tests.rs"]
mod performance_tests;
