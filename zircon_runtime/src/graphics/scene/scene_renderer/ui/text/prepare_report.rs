use super::font_id_report::ScreenSpaceUiTextFontIdReport;
use super::native_bitmap_atlas::NativeBitmapAtlasPrepareReport;
use super::sdf_fallback::ScreenSpaceUiTextSdfFallbackReport;
use super::{ResolvedScreenSpaceUiTextBatches, ScreenSpaceUiNativePrepareReport};
use crate::graphics::scene::scene_renderer::ui::atlas_renderer::GlyphAtlasBitmapRendererPrepareReport;
use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextBatch;
use crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasCacheReport;
use crate::graphics::scene::scene_renderer::ui::sdf_render::ScreenSpaceUiSdfPrepareReport;
use crate::graphics::text::font::MissingGlyphDiagnosticsReport;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextPrepareReport {
    pub(super) input_auto_text_batch_count: usize,
    pub(super) input_native_text_batch_count: usize,
    pub(super) input_sdf_text_batch_count: usize,
    pub(super) resolved_native_text_batch_count: usize,
    pub(super) resolved_sdf_text_batch_count: usize,
    pub(super) sdf_fallback: ScreenSpaceUiTextSdfFallbackReport,
    pub(crate) native_font_ids: ScreenSpaceUiTextFontIdReport,
    pub(super) missing_glyphs: MissingGlyphDiagnosticsReport,
    pub(crate) raster_upload: ScreenSpaceUiTextRasterUploadReport,
    pub(super) native_bitmap_atlas: NativeBitmapAtlasPrepareReport,
    pub(super) bitmap_atlas_renderer: GlyphAtlasBitmapRendererPrepareReport,
    pub(super) sdf_atlas: SdfAtlasCacheReport,
    pub(super) sdf_renderer: ScreenSpaceUiSdfPrepareReport,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextRasterUploadReport {
    pub(crate) visible_raster_glyph_count: usize,
    pub(crate) source_image_count: usize,
    pub(super) missing_raster_image_count: usize,
    pub(super) approximate_raster_image_count: usize,
    pub(super) source_cache_hit_count: usize,
    pub(super) source_cache_approximate_hit_count: usize,
    pub(super) source_cache_miss_count: usize,
    pub(super) source_cache_insert_count: usize,
    pub(super) worker_request_submitted_count: usize,
    pub(crate) worker_request_pending_count: usize,
    pub(crate) worker_request_unavailable_count: usize,
    pub(crate) worker_request_failed_count: usize,
    pub(super) upload_command_count: usize,
    pub(super) upload_copy_count: usize,
    pub(super) upload_byte_len: usize,
    pub(super) renderer_upload_request_count: usize,
    pub(super) renderer_upload_byte_len: usize,
    pub(super) renderer_upload_requeued_count: usize,
    pub(super) renderer_upload_failure_count: usize,
    pub(super) renderer_upload_ready_to_write_texture: bool,
}

pub(super) fn text_prepare_report(
    auto_texts: &[ScreenSpaceUiTextBatch],
    native_texts: &[ScreenSpaceUiTextBatch],
    sdf_texts: &[ScreenSpaceUiTextBatch],
    resolved_texts: &ResolvedScreenSpaceUiTextBatches,
    sdf_fallback: ScreenSpaceUiTextSdfFallbackReport,
    native_prepare: ScreenSpaceUiNativePrepareReport,
    missing_glyphs: MissingGlyphDiagnosticsReport,
    bitmap_atlas_renderer: GlyphAtlasBitmapRendererPrepareReport,
    sdf_atlas: SdfAtlasCacheReport,
    sdf_renderer: ScreenSpaceUiSdfPrepareReport,
) -> ScreenSpaceUiTextPrepareReport {
    let raster_upload =
        text_raster_upload_report(&native_prepare.bitmap_atlas, &bitmap_atlas_renderer);
    ScreenSpaceUiTextPrepareReport {
        input_auto_text_batch_count: auto_texts.len(),
        input_native_text_batch_count: native_texts.len(),
        input_sdf_text_batch_count: sdf_texts.len(),
        resolved_native_text_batch_count: resolved_texts.native_texts().len(),
        resolved_sdf_text_batch_count: resolved_texts.sdf_texts().len(),
        sdf_fallback,
        native_font_ids: native_prepare.font_ids,
        missing_glyphs,
        raster_upload,
        native_bitmap_atlas: native_prepare.bitmap_atlas,
        bitmap_atlas_renderer,
        sdf_atlas,
        sdf_renderer,
    }
}

fn text_raster_upload_report(
    native_bitmap_atlas: &NativeBitmapAtlasPrepareReport,
    bitmap_atlas_renderer: &GlyphAtlasBitmapRendererPrepareReport,
) -> ScreenSpaceUiTextRasterUploadReport {
    ScreenSpaceUiTextRasterUploadReport {
        visible_raster_glyph_count: native_bitmap_atlas.visible_raster_glyph_count,
        source_image_count: native_bitmap_atlas.source_image_count,
        missing_raster_image_count: native_bitmap_atlas.missing_raster_image_count,
        approximate_raster_image_count: native_bitmap_atlas.approximate_raster_image_count,
        source_cache_hit_count: native_bitmap_atlas.source_cache.hit_count,
        source_cache_approximate_hit_count: native_bitmap_atlas.source_cache.approximate_hit_count,
        source_cache_miss_count: native_bitmap_atlas.source_cache.miss_count,
        source_cache_insert_count: native_bitmap_atlas.source_cache.insert_count,
        worker_request_submitted_count: native_bitmap_atlas
            .source_cache
            .worker_request_submitted_count,
        worker_request_pending_count: native_bitmap_atlas
            .source_cache
            .worker_request_pending_count,
        worker_request_unavailable_count: native_bitmap_atlas
            .source_cache
            .worker_request_unavailable_count,
        worker_request_failed_count: native_bitmap_atlas.source_cache.worker_request_failed_count,
        upload_command_count: native_bitmap_atlas.submission.upload_command_count,
        upload_copy_count: native_bitmap_atlas.submission.upload_copy_count,
        upload_byte_len: native_bitmap_atlas.submission.upload_byte_len,
        renderer_upload_request_count: bitmap_atlas_renderer.upload_request_count,
        renderer_upload_byte_len: bitmap_atlas_renderer.upload_byte_len,
        renderer_upload_requeued_count: bitmap_atlas_renderer.upload_requeued_count,
        renderer_upload_failure_count: bitmap_atlas_renderer.upload_failure_count,
        renderer_upload_ready_to_write_texture: bitmap_atlas_renderer.upload_ready_to_write_texture,
    }
}
