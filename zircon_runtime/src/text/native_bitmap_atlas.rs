use glyphon::{cosmic_text::LayoutGlyph, Color, FontSystem, SwashContent, TextArea, TextBounds};

use crate::core::math::UVec2;
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapPlaceholderGlyph,
    GlyphAtlasBitmapPlaceholderMode, GlyphAtlasBitmapRenderSubmissionPlan,
    GlyphAtlasBitmapRenderSubmissionReport, GlyphAtlasBitmapRetryFrameState,
    GlyphAtlasBitmapRetryFrameStateReport, GlyphAtlasBitmapRetryFrameSubmissionReport,
    GlyphAtlasBitmapSource, GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasFormat, GlyphAtlasSet,
    GlyphAtlasStorageFormat,
};
use crate::text::font::FontDatabase;
use crate::text::parallel::raster_pool::{TextRasterWorkTarget, TextRasterWorkerPool};

mod handoff;
mod retry_frame;
mod source_cache;
mod storage;

pub(crate) use handoff::{
    native_bitmap_atlas_first_frame_degradation_for_report,
    native_bitmap_atlas_glyphon_fallback_reason_for_report, native_bitmap_atlas_handoff_for_report,
    NativeBitmapAtlasFirstFrameDegradation, NativeBitmapAtlasGlyphonFallbackReason,
    NativeBitmapAtlasHandoff,
};
use retry_frame::native_bitmap_atlas_retry_frame;

pub(crate) use source_cache::{
    NativeBitmapAtlasSourceCache, NativeBitmapAtlasSourceCacheFrameReport,
};
use storage::{
    native_bitmap_atlas_has_mixed_storage_formats, native_bitmap_atlas_storage_submissions,
    single_native_bitmap_atlas_storage_format, NativeBitmapAtlasStorageSubmission,
};

const NATIVE_BITMAP_ATLAS_WORK_PAGE_GENERATION: u64 = 0;

pub(crate) struct NativeBitmapAtlasFrame {
    pub(crate) submission: GlyphAtlasBitmapRenderSubmissionPlan,
    source_images: Vec<NativeBitmapAtlasSourceImage>,
    frame_index: u64,
    viewport_size: UVec2,
    clip_rect: GlyphAtlasScreenRect,
    visible_raster_glyph_count: usize,
    missing_raster_image_count: usize,
    approximate_raster_image_count: usize,
    unsupported_glyph_count: usize,
    clipped_glyph_count: usize,
    background_composite_glyph_count: usize,
    missing_background_composite_glyph_count: usize,
    source_cache: NativeBitmapAtlasSourceCacheFrameReport,
    retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport,
    retry_state: GlyphAtlasBitmapRetryFrameStateReport,
    discarded_stale_retry_glyph_count: usize,
    face_epoch: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeBitmapAtlasPrepareReport {
    pub(crate) frame_index: u64,
    pub(crate) visible_raster_glyph_count: usize,
    pub(crate) source_image_count: usize,
    pub(crate) missing_raster_image_count: usize,
    pub(crate) approximate_raster_image_count: usize,
    pub(crate) unsupported_glyph_count: usize,
    pub(crate) clipped_glyph_count: usize,
    pub(crate) atlas_storage_format: Option<GlyphAtlasStorageFormat>,
    pub(crate) mixed_atlas_storage_format: bool,
    pub(crate) storage_submission_count: usize,
    pub(crate) storage_submission_visible_glyph_count: usize,
    pub(crate) mixed_storage_replacement_ready: bool,
    pub(crate) requires_background_composite: bool,
    pub(crate) background_composite_replacement_ready: bool,
    pub(crate) background_composite_glyph_count: usize,
    pub(crate) missing_background_composite_glyph_count: usize,
    pub(crate) source_cache: NativeBitmapAtlasSourceCacheFrameReport,
    pub(crate) retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport,
    pub(crate) retry_state: GlyphAtlasBitmapRetryFrameStateReport,
    pub(crate) discarded_stale_retry_glyph_count: usize,
    pub(crate) glyphon_fallback_reason: Option<NativeBitmapAtlasGlyphonFallbackReason>,
    pub(crate) first_frame_degradation: Option<NativeBitmapAtlasFirstFrameDegradation>,
    pub(crate) replaces_glyphon: bool,
    pub(crate) submission: GlyphAtlasBitmapRenderSubmissionReport,
}

pub(crate) struct NativeBitmapAtlasTextArea<'a, 'text> {
    text_area: &'a TextArea<'text>,
    background_color: Option<[f32; 4]>,
}

#[derive(Clone)]
struct NativeBitmapAtlasSourceImage {
    source: GlyphAtlasBitmapSource,
    bytes: Vec<u8>,
    face_epoch: u64,
}

#[derive(Clone, Copy)]
struct NativeBitmapAtlasPendingPlaceholder {
    format: GlyphAtlasFormat,
    screen_rect: GlyphAtlasScreenRect,
    retry_frame_index: u64,
}

#[derive(Clone, Copy)]
struct NativeBitmapGlyphImage {
    x: i32,
    y: i32,
    line_y: f32,
    top: i16,
    left: i16,
    width: u16,
    height: u16,
    format: GlyphAtlasFormat,
    scale_factor: f32,
    source_byte_len: usize,
    foreground_color: [f32; 4],
    background_color: [f32; 4],
}

impl NativeBitmapAtlasFrame {
    pub(crate) fn replaces_glyphon(&self) -> bool {
        self.source_coverage_supports_replacement()
            && self.submission.run.allocation_failures.is_empty()
            && self.submission.gpu_draw.visible_glyph_count == self.visible_raster_glyph_count
            && self.atlas_storage_format().is_some()
            && self.background_composite_supports_replacement()
    }

    pub(crate) fn source_bytes(&self) -> Vec<GlyphAtlasBitmapUploadSourceBytes<'_>> {
        self.source_images
            .iter()
            .enumerate()
            .map(|(source_index, image)| {
                GlyphAtlasBitmapUploadSourceBytes::with_face_epoch(
                    source_index,
                    &image.bytes,
                    image.face_epoch,
                )
            })
            .collect()
    }

    pub(crate) fn face_validity(&self) -> GlyphAtlasBitmapFaceValidity {
        glyph_atlas_bitmap_face_validity_for_epoch(
            self.source_images.iter().map(|image| image.face_epoch),
            self.face_epoch,
        )
    }

    pub(crate) fn storage_submissions(&self) -> Vec<NativeBitmapAtlasStorageSubmission> {
        native_bitmap_atlas_storage_submissions(
            &self.submission.run.atlas,
            &self.source_images,
            self.frame_index,
            self.viewport_size,
            self.clip_rect,
            self.face_epoch,
        )
    }

    pub(crate) fn atlas_layer_count(&self) -> u32 {
        self.submission
            .gpu_draw
            .vertices
            .iter()
            .map(|vertex| vertex.page_index)
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(1)
    }

    pub(crate) fn atlas_storage_format(&self) -> Option<GlyphAtlasStorageFormat> {
        single_native_bitmap_atlas_storage_format(
            self.submission
                .run
                .glyphs
                .iter()
                .map(|glyph| glyph.page_key.format.storage_format()),
        )
    }

    pub(crate) fn prepare_report(&self) -> NativeBitmapAtlasPrepareReport {
        let storage_submissions = self.storage_submissions();
        let storage_submission_visible_glyph_count = storage_submissions
            .iter()
            .map(NativeBitmapAtlasStorageSubmission::visible_glyph_count)
            .sum();
        let storage_submissions_replace_glyphon =
            self.storage_submissions_replace_glyphon(&storage_submissions);
        let background_composite_replacement_ready = self.background_composite_replacement_ready();
        let mut report = NativeBitmapAtlasPrepareReport {
            frame_index: self.frame_index,
            visible_raster_glyph_count: self.visible_raster_glyph_count,
            source_image_count: self.source_images.len(),
            missing_raster_image_count: self.missing_raster_image_count,
            approximate_raster_image_count: self.approximate_raster_image_count,
            unsupported_glyph_count: self.unsupported_glyph_count,
            clipped_glyph_count: self.clipped_glyph_count,
            atlas_storage_format: self.atlas_storage_format(),
            mixed_atlas_storage_format: native_bitmap_atlas_has_mixed_storage_formats(
                self.submission
                    .run
                    .glyphs
                    .iter()
                    .map(|glyph| glyph.page_key.format.storage_format()),
            ),
            storage_submission_count: storage_submissions.len(),
            storage_submission_visible_glyph_count,
            mixed_storage_replacement_ready: storage_submissions.len() > 1
                && storage_submissions_replace_glyphon,
            requires_background_composite: self.submission.gpu_draw.requires_background_composite,
            background_composite_replacement_ready,
            background_composite_glyph_count: self.background_composite_glyph_count,
            missing_background_composite_glyph_count: self.missing_background_composite_glyph_count,
            source_cache: self.source_cache,
            retry_submission: self.retry_submission,
            retry_state: self.retry_state,
            discarded_stale_retry_glyph_count: self.discarded_stale_retry_glyph_count,
            glyphon_fallback_reason: None,
            first_frame_degradation: None,
            replaces_glyphon: self.replaces_glyphon(),
            submission: self.submission.submission_report(),
        };
        report.glyphon_fallback_reason =
            native_bitmap_atlas_glyphon_fallback_reason_for_report(&report);
        report.first_frame_degradation =
            native_bitmap_atlas_first_frame_degradation_for_report(&report);
        report
    }

    fn source_coverage_supports_replacement(&self) -> bool {
        self.visible_raster_glyph_count > 0
            && self.missing_raster_image_count == 0
            && self.unsupported_glyph_count == 0
            && self.clipped_glyph_count <= self.visible_raster_glyph_count
            && self.source_images.len() == self.visible_raster_glyph_count
    }

    fn background_composite_supports_replacement(&self) -> bool {
        !self.submission.gpu_draw.requires_background_composite
            || self.background_composite_replacement_ready()
    }

    fn background_composite_replacement_ready(&self) -> bool {
        self.submission.gpu_draw.requires_background_composite
            && self.background_composite_glyph_count > 0
            && self.missing_background_composite_glyph_count == 0
    }

    fn storage_submissions_replace_glyphon(
        &self,
        storage_submissions: &[NativeBitmapAtlasStorageSubmission],
    ) -> bool {
        if !self.source_coverage_supports_replacement()
            || !self.background_composite_supports_replacement()
        {
            return false;
        }

        let source_count = storage_submissions
            .iter()
            .map(NativeBitmapAtlasStorageSubmission::source_image_count)
            .sum::<usize>();
        let visible_count = storage_submissions
            .iter()
            .map(NativeBitmapAtlasStorageSubmission::visible_glyph_count)
            .sum::<usize>();
        let has_failures = storage_submissions
            .iter()
            .any(NativeBitmapAtlasStorageSubmission::has_allocation_failures);

        source_count == self.visible_raster_glyph_count
            && visible_count == self.visible_raster_glyph_count
            && !has_failures
    }
}

impl<'a, 'text> NativeBitmapAtlasTextArea<'a, 'text> {
    pub(crate) fn new(text_area: &'a TextArea<'text>, background_color: Option<[f32; 4]>) -> Self {
        Self {
            text_area,
            background_color,
        }
    }
}

pub(crate) fn native_bitmap_atlas_frame(
    font_system: &mut FontSystem,
    font_database: &FontDatabase,
    raster_worker_pool: Option<&TextRasterWorkerPool>,
    source_cache: &mut NativeBitmapAtlasSourceCache,
    retry_state: &mut GlyphAtlasBitmapRetryFrameState,
    atlas: GlyphAtlasSet,
    viewport_size: UVec2,
    frame_index: u64,
    text_areas: &[NativeBitmapAtlasTextArea<'_, '_>],
) -> NativeBitmapAtlasFrame {
    source_cache.begin_frame();
    let worker_target = native_bitmap_atlas_worker_target(source_cache);
    if let Some(raster_worker_pool) = raster_worker_pool {
        let completion_drain = raster_worker_pool.drain_completed_for_target(worker_target);
        source_cache.apply_worker_completion_drain(completion_drain);
    }

    let mut source_images = Vec::new();
    let mut visible_raster_glyph_count: usize = 0;
    let mut missing_raster_image_count: usize = 0;
    let mut approximate_raster_image_count: usize = 0;
    let mut unsupported_glyph_count: usize = 0;
    let mut clipped_glyph_count: usize = 0;
    let mut background_composite_glyph_count: usize = 0;
    let mut missing_background_composite_glyph_count: usize = 0;
    let mut pending_placeholders = Vec::new();

    for bitmap_text_area in text_areas {
        let text_area = bitmap_text_area.text_area;
        let default_color = unpack_color(text_area.default_color);
        for run in text_area.buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical = glyph.physical((text_area.left, text_area.top), text_area.scale);
                let image = match source_cache.cached_image(physical.cache_key) {
                    Some(image) => image,
                    None => {
                        if let Some(approximate_image) =
                            source_cache.approximate_cached_image(physical.cache_key)
                        {
                            approximate_raster_image_count =
                                approximate_raster_image_count.saturating_add(1);
                            source_cache.request_worker_image(
                                font_system,
                                font_database,
                                raster_worker_pool,
                                worker_target,
                                physical.cache_key,
                            );
                            approximate_image
                        } else {
                            let already_pending =
                                source_cache.has_pending_worker_request(physical.cache_key);
                            let submitted = source_cache
                                .request_worker_image(
                                    font_system,
                                    font_database,
                                    raster_worker_pool,
                                    worker_target,
                                    physical.cache_key,
                                )
                                .is_some();
                            if already_pending || submitted {
                                pending_placeholders.push(native_bitmap_atlas_pending_placeholder(
                                    glyph,
                                    run.line_top,
                                    run.line_height,
                                    text_area,
                                    frame_index,
                                ));
                            }
                            missing_raster_image_count += 1;
                            continue;
                        }
                    }
                };
                if image.width == 0 || image.height == 0 {
                    continue;
                }
                let screen_rect = native_bitmap_atlas_screen_rect(
                    physical.x,
                    physical.y,
                    run.line_y,
                    image.top,
                    image.left,
                    image.width,
                    image.height,
                    text_area.scale,
                );
                let Some(clipped_rect) =
                    text_bounds_clipped_screen_rect(text_area.bounds, screen_rect)
                else {
                    continue;
                };
                visible_raster_glyph_count += 1;

                let Some(format) = native_bitmap_atlas_format(image.content) else {
                    unsupported_glyph_count += 1;
                    continue;
                };
                if native_bitmap_atlas_format_requires_background_composite(format) {
                    background_composite_glyph_count += 1;
                    if bitmap_text_area.background_color.is_none() {
                        missing_background_composite_glyph_count += 1;
                    }
                }

                let glyph_image = NativeBitmapGlyphImage {
                    x: physical.x,
                    y: physical.y,
                    line_y: run.line_y,
                    top: image.top,
                    left: image.left,
                    width: image.width,
                    height: image.height,
                    format,
                    scale_factor: text_area.scale,
                    source_byte_len: image.bytes.len(),
                    foreground_color: native_bitmap_atlas_foreground_color(
                        format,
                        glyph.color_opt.map(unpack_color).unwrap_or(default_color),
                    ),
                    background_color: native_bitmap_atlas_background_color(
                        format,
                        bitmap_text_area.background_color,
                    ),
                };
                if let Some(clipped_source) =
                    native_bitmap_atlas_source_from_image(glyph_image, clipped_rect, image.bytes)
                {
                    clipped_glyph_count += usize::from(clipped_source.was_clipped);
                    source_images.push(NativeBitmapAtlasSourceImage {
                        source: clipped_source.source,
                        bytes: clipped_source.bytes,
                        face_epoch: source_cache.face_epoch(),
                    });
                }
            }
        }
    }

    let clip_rect = GlyphAtlasScreenRect::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let retry_frame = native_bitmap_atlas_retry_frame(
        retry_state,
        atlas,
        source_images,
        frame_index,
        viewport_size,
        clip_rect,
    );
    let mut submission = retry_frame.submission;
    let source_images = retry_frame.source_images;
    append_native_bitmap_atlas_pending_placeholders(
        &mut submission,
        source_images.len(),
        pending_placeholders,
        clip_rect,
    );

    NativeBitmapAtlasFrame {
        submission,
        source_images,
        frame_index,
        viewport_size,
        clip_rect,
        visible_raster_glyph_count,
        missing_raster_image_count,
        approximate_raster_image_count,
        unsupported_glyph_count,
        clipped_glyph_count,
        background_composite_glyph_count,
        missing_background_composite_glyph_count,
        source_cache: source_cache.frame_report(),
        retry_submission: retry_frame.retry_submission,
        retry_state: retry_frame.retry_state,
        discarded_stale_retry_glyph_count: retry_frame.discarded_stale_retry_glyph_count,
        face_epoch: source_cache.face_epoch(),
    }
}

fn native_bitmap_atlas_worker_target(
    source_cache: &NativeBitmapAtlasSourceCache,
) -> TextRasterWorkTarget {
    TextRasterWorkTarget::new(
        NATIVE_BITMAP_ATLAS_WORK_PAGE_GENERATION,
        source_cache.face_epoch(),
    )
}

fn native_bitmap_atlas_pending_placeholder(
    glyph: &LayoutGlyph,
    line_top: f32,
    line_height: f32,
    text_area: &TextArea<'_>,
    frame_index: u64,
) -> NativeBitmapAtlasPendingPlaceholder {
    let scale = text_area.scale;
    let x_offset = glyph.font_size * glyph.x_offset;
    let x = (glyph.x + x_offset).mul_add(scale, text_area.left);
    let y = line_top.mul_add(scale, text_area.top);
    NativeBitmapAtlasPendingPlaceholder {
        format: GlyphAtlasFormat::AlphaMask,
        screen_rect: GlyphAtlasScreenRect::new(
            x,
            y,
            (glyph.w * scale).abs().max(1.0),
            (line_height * scale).abs().max(1.0),
        ),
        retry_frame_index: frame_index.saturating_add(1),
    }
}

fn append_native_bitmap_atlas_pending_placeholders(
    submission: &mut GlyphAtlasBitmapRenderSubmissionPlan,
    source_image_count: usize,
    placeholders: Vec<NativeBitmapAtlasPendingPlaceholder>,
    clip_rect: GlyphAtlasScreenRect,
) {
    submission.append_placeholder_glyphs(
        placeholders
            .into_iter()
            .enumerate()
            .map(
                |(placeholder_index, placeholder)| GlyphAtlasBitmapPlaceholderGlyph {
                    source_index: source_image_count.saturating_add(placeholder_index),
                    format: placeholder.format,
                    screen_rect: placeholder.screen_rect,
                    retry_frame_index: placeholder.retry_frame_index,
                    mode: GlyphAtlasBitmapPlaceholderMode::TransparentQuad,
                },
            ),
        clip_rect,
    );
}

pub(crate) fn native_bitmap_atlas_idle_prepare_report(
    source_cache: &mut NativeBitmapAtlasSourceCache,
    retry_state: &mut GlyphAtlasBitmapRetryFrameState,
) -> NativeBitmapAtlasPrepareReport {
    NativeBitmapAtlasPrepareReport {
        source_cache: source_cache.discard_all_for_idle_frame(),
        retry_state: retry_state.take_report(),
        ..NativeBitmapAtlasPrepareReport::default()
    }
}

pub(crate) fn bitmap_atlas_page_size() -> UVec2 {
    UVec2::new(512, 512)
}

struct NativeBitmapAtlasClippedSource {
    source: GlyphAtlasBitmapSource,
    bytes: Vec<u8>,
    was_clipped: bool,
}

fn native_bitmap_atlas_source_from_image(
    image: NativeBitmapGlyphImage,
    clipped_rect: GlyphAtlasScreenRect,
    source_bytes: Vec<u8>,
) -> Option<NativeBitmapAtlasClippedSource> {
    let content_size = UVec2::new(u32::from(image.width), u32::from(image.height));
    if content_size.x == 0 || content_size.y == 0 {
        return None;
    }
    if image.source_byte_len != source_bytes.len() {
        return None;
    }

    let screen_rect = native_bitmap_atlas_screen_rect(
        image.x,
        image.y,
        image.line_y,
        image.top,
        image.left,
        image.width,
        image.height,
        image.scale_factor,
    );
    let crop = native_bitmap_atlas_crop_from_clip(screen_rect, clipped_rect, content_size)?;
    let bytes = crop_native_bitmap_source_bytes(source_bytes, content_size, image.format, crop)?;
    let source = GlyphAtlasBitmapSource {
        format: image.format,
        content_size: crop.size,
        screen_rect: clipped_rect,
        foreground_color: image.foreground_color,
        background_color: image.background_color,
        source_byte_len: bytes.len(),
    };

    Some(NativeBitmapAtlasClippedSource {
        source,
        bytes,
        was_clipped: crop.was_clipped,
    })
}

fn glyph_atlas_bitmap_face_validity_for_epoch(
    source_face_epochs: impl IntoIterator<Item = u64>,
    current_face_epoch: u64,
) -> GlyphAtlasBitmapFaceValidity {
    if source_face_epochs
        .into_iter()
        .all(|face_epoch| face_epoch == current_face_epoch)
    {
        GlyphAtlasBitmapFaceValidity::Valid
    } else {
        GlyphAtlasBitmapFaceValidity::Invalidated
    }
}

fn native_bitmap_atlas_format(content: SwashContent) -> Option<GlyphAtlasFormat> {
    match content {
        SwashContent::Mask => Some(GlyphAtlasFormat::AlphaMask),
        SwashContent::Color => Some(GlyphAtlasFormat::Color),
        SwashContent::SubpixelMask => Some(GlyphAtlasFormat::SubpixelMask),
    }
}

fn native_bitmap_atlas_foreground_color(
    format: GlyphAtlasFormat,
    text_color: [f32; 4],
) -> [f32; 4] {
    match format {
        GlyphAtlasFormat::Color => [1.0, 1.0, 1.0, 1.0],
        GlyphAtlasFormat::AlphaMask | GlyphAtlasFormat::SubpixelMask => text_color,
        GlyphAtlasFormat::Sdf | GlyphAtlasFormat::Msdf => text_color,
    }
}

fn native_bitmap_atlas_background_color(
    format: GlyphAtlasFormat,
    text_background_color: Option<[f32; 4]>,
) -> [f32; 4] {
    match format {
        GlyphAtlasFormat::SubpixelMask => text_background_color.unwrap_or([0.0, 0.0, 0.0, 1.0]),
        GlyphAtlasFormat::AlphaMask
        | GlyphAtlasFormat::Color
        | GlyphAtlasFormat::Sdf
        | GlyphAtlasFormat::Msdf => [0.0, 0.0, 0.0, 1.0],
    }
}

fn native_bitmap_atlas_format_requires_background_composite(format: GlyphAtlasFormat) -> bool {
    matches!(format, GlyphAtlasFormat::SubpixelMask)
}

#[derive(Clone, Copy)]
struct NativeBitmapAtlasCrop {
    left: u32,
    top: u32,
    size: UVec2,
    was_clipped: bool,
}

fn native_bitmap_atlas_screen_rect(
    x: i32,
    y: i32,
    line_y: f32,
    top: i16,
    left: i16,
    width: u16,
    height: u16,
    scale_factor: f32,
) -> GlyphAtlasScreenRect {
    GlyphAtlasScreenRect::new(
        (x + i32::from(left)) as f32,
        ((line_y * scale_factor).round() as i32 + y - i32::from(top)) as f32,
        f32::from(width),
        f32::from(height),
    )
}

fn text_bounds_clipped_screen_rect(
    bounds: TextBounds,
    rect: GlyphAtlasScreenRect,
) -> Option<GlyphAtlasScreenRect> {
    rect.clipped_to(GlyphAtlasScreenRect::new(
        bounds.left as f32,
        bounds.top as f32,
        (bounds.right - bounds.left).max(0) as f32,
        (bounds.bottom - bounds.top).max(0) as f32,
    ))
}

fn native_bitmap_atlas_crop_from_clip(
    screen_rect: GlyphAtlasScreenRect,
    clipped_rect: GlyphAtlasScreenRect,
    source_size: UVec2,
) -> Option<NativeBitmapAtlasCrop> {
    let left = rounded_non_negative_u32(clipped_rect.x - screen_rect.x)?;
    let top = rounded_non_negative_u32(clipped_rect.y - screen_rect.y)?;
    let width = rounded_non_negative_u32(clipped_rect.width)?;
    let height = rounded_non_negative_u32(clipped_rect.height)?;
    if width == 0 || height == 0 {
        return None;
    }
    if left.checked_add(width)? > source_size.x || top.checked_add(height)? > source_size.y {
        return None;
    }

    Some(NativeBitmapAtlasCrop {
        left,
        top,
        size: UVec2::new(width, height),
        was_clipped: left != 0 || top != 0 || width != source_size.x || height != source_size.y,
    })
}

fn rounded_non_negative_u32(value: f32) -> Option<u32> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }

    u32::try_from(value.round() as i64).ok()
}

fn crop_native_bitmap_source_bytes(
    source_bytes: Vec<u8>,
    source_size: UVec2,
    format: GlyphAtlasFormat,
    crop: NativeBitmapAtlasCrop,
) -> Option<Vec<u8>> {
    if !crop.was_clipped {
        return Some(source_bytes);
    }

    let bytes_per_pixel = format.storage_format().bytes_per_pixel() as usize;
    let source_stride = source_size.x as usize * bytes_per_pixel;
    let crop_stride = crop.size.x as usize * bytes_per_pixel;
    let expected_byte_len = source_stride.checked_mul(source_size.y as usize)?;
    if source_bytes.len() != expected_byte_len {
        return None;
    }

    let mut cropped = Vec::with_capacity(crop_stride.saturating_mul(crop.size.y as usize));
    for row in 0..crop.size.y as usize {
        let source_row = crop.top as usize + row;
        let source_start = source_row
            .saturating_mul(source_stride)
            .saturating_add(crop.left as usize * bytes_per_pixel);
        let source_end = source_start.saturating_add(crop_stride);
        cropped.extend_from_slice(source_bytes.get(source_start..source_end)?);
    }
    Some(cropped)
}

fn unpack_color(color: Color) -> [f32; 4] {
    let [r, g, b, a] = color.as_rgba();
    [
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    ]
}

#[cfg(test)]
mod tests;
