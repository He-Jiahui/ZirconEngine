use std::collections::HashMap;
use std::sync::Arc;

use crate::core::math::UVec2;
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapPlaceholderGlyph,
    GlyphAtlasBitmapPlaceholderMode, GlyphAtlasBitmapRenderSubmissionPlan,
    GlyphAtlasBitmapRetryFrameState, GlyphAtlasBitmapRetryFrameStateReport,
    GlyphAtlasBitmapRetryFrameSubmissionReport, GlyphAtlasBitmapSource,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasFormat, GlyphAtlasSet, GlyphAtlasStorageFormat,
    GlyphRasterKey,
};
use crate::text::font::FontDatabase;
use crate::text::parallel::raster_pool::{TextRasterCompletionDrainBudget, TextRasterWorkerPool};

use super::glyph_run::NativeBitmapAtlasGlyphRun;
use super::handoff::{
    native_bitmap_atlas_degradation_reason_for_report,
    native_bitmap_atlas_first_frame_degradation_for_report, native_bitmap_atlas_handoff_for_report,
};
use super::report::NativeBitmapAtlasPrepareReport;
use super::retry_frame::native_bitmap_atlas_retry_frame;
use super::source_cache::native_bitmap_atlas_raster_key_for_content;
use super::source_cache::{
    NativeBitmapAtlasCachedGlyphImage, NativeBitmapAtlasSourceCache,
    NativeBitmapAtlasSourceCacheFrameReport, NativeBitmapAtlasWorkerRequestStatus,
};
use super::source_image::{
    NativeBitmapGlyphImage, glyph_atlas_bitmap_face_validity_for_epoch,
    native_bitmap_atlas_background_color, native_bitmap_atlas_foreground_color,
    native_bitmap_atlas_format, native_bitmap_atlas_format_requires_background_composite,
    native_bitmap_atlas_screen_rect, native_bitmap_atlas_source_from_image,
};
use super::storage::{
    native_bitmap_atlas_has_mixed_storage_formats, native_bitmap_atlas_storage_resource_count,
    single_native_bitmap_atlas_format, single_native_bitmap_atlas_storage_format,
};
use super::{NativeBitmapAtlasReadinessChangeReceipt, NativeBitmapAtlasReadinessGeneration};

pub(super) const NATIVE_BITMAP_ATLAS_MAX_RASTER_COMPLETIONS_PER_FRAME: usize = 256;
pub(super) const NATIVE_BITMAP_ATLAS_MAX_RASTER_COMPLETION_BYTES_PER_FRAME: usize = 2 * 1024 * 1024;

pub(crate) struct NativeBitmapAtlasFrame {
    pub(crate) submission: GlyphAtlasBitmapRenderSubmissionPlan,
    pub(super) source_images: Vec<NativeBitmapAtlasSourceImage>,
    pub(super) frame_index: u64,
    pub(super) viewport_size: UVec2,
    pub(super) clip_rect: GlyphAtlasScreenRect,
    pub(super) visible_raster_glyph_count: usize,
    pub(super) missing_raster_image_count: usize,
    pub(super) visible_missing_raster_image_count: usize,
    pub(super) approximate_raster_image_count: usize,
    pub(super) unsupported_glyph_count: usize,
    pub(super) clipped_glyph_count: usize,
    pub(super) background_composite_glyph_count: usize,
    pub(super) missing_background_composite_glyph_count: usize,
    pub(super) source_cache: NativeBitmapAtlasSourceCacheFrameReport,
    pub(super) retry_submission: GlyphAtlasBitmapRetryFrameSubmissionReport,
    pub(super) retry_state: GlyphAtlasBitmapRetryFrameStateReport,
    pub(super) discarded_stale_retry_glyph_count: usize,
    pub(super) face_epoch: u64,
    readiness_changes: NativeBitmapAtlasReadinessChangeReceipt,
}

#[derive(Clone)]
pub(super) struct NativeBitmapAtlasSourceImage {
    pub(super) source: GlyphAtlasBitmapSource,
    pub(super) bytes: Arc<[u8]>,
    pub(super) face_epoch: u64,
}

#[derive(Clone, Copy)]
struct NativeBitmapAtlasPendingPlaceholder {
    format: GlyphAtlasFormat,
    screen_rect: GlyphAtlasScreenRect,
    retry_frame_index: u64,
}

enum NativeBitmapAtlasGlyphReadiness {
    Exact {
        image: NativeBitmapAtlasCachedGlyphImage,
        persistent_raster_key: Option<GlyphRasterKey>,
    },
    Approximate {
        image: NativeBitmapAtlasCachedGlyphImage,
        worker_request: Option<NativeBitmapAtlasWorkerRequestStatus>,
    },
    Missing {
        worker_request: Option<NativeBitmapAtlasWorkerRequestStatus>,
    },
}

impl NativeBitmapAtlasFrame {
    pub(crate) fn supports_native_submission(&self) -> bool {
        self.source_coverage_supports_replacement()
            && self.submission.run.allocation_failures.is_empty()
            && self.submission.gpu_draw.visible_glyph_count == self.visible_raster_glyph_count
            && self.atlas_format().is_some()
            && self.background_composite_supports_replacement()
    }

    pub(crate) fn source_bytes(
        &self,
    ) -> impl Iterator<Item = GlyphAtlasBitmapUploadSourceBytes<'_>> + '_ {
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
    }

    pub(crate) fn face_validity(&self) -> GlyphAtlasBitmapFaceValidity {
        glyph_atlas_bitmap_face_validity_for_epoch(
            self.source_images.iter().map(|image| image.face_epoch),
            self.face_epoch,
        )
    }

    pub(crate) fn readiness_generation(&self) -> NativeBitmapAtlasReadinessGeneration {
        self.readiness_changes().generation()
    }

    pub(crate) fn readiness_changes(&self) -> &NativeBitmapAtlasReadinessChangeReceipt {
        &self.readiness_changes
    }

    pub(crate) fn atlas_layer_count(&self) -> u32 {
        self.submission
            .gpu_draw
            .instances
            .iter()
            .map(|instance| instance.page_index)
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

    pub(crate) fn atlas_format(&self) -> Option<GlyphAtlasFormat> {
        single_native_bitmap_atlas_format(
            self.submission
                .run
                .glyphs
                .iter()
                .map(|glyph| glyph.page_key.format),
        )
    }

    pub(crate) fn canonical_frame_plan_count(&self) -> usize {
        usize::from(!self.submission.gpu_draw.instances.is_empty())
    }

    pub(crate) fn storage_resource_count(&self) -> usize {
        native_bitmap_atlas_storage_resource_count(
            self.submission
                .run
                .glyphs
                .iter()
                .map(|glyph| glyph.page_key.format),
        )
    }

    pub(crate) fn ordered_draw_segment_count(&self) -> usize {
        let mut previous_format = None;
        let mut segment_count = 0;
        for command in &self.submission.gpu_draw.draw_commands {
            let format = command.key.page_key.format;
            if previous_format != Some(format) {
                segment_count += 1;
                previous_format = Some(format);
            }
        }
        segment_count
    }

    pub(crate) fn prepare_report(&self) -> NativeBitmapAtlasPrepareReport {
        crate::profile_counter!(
            "runtime",
            "ui_text.native_raster_plan.readiness_generation",
            self.readiness_generation().value()
        );
        crate::profile_counter!(
            "runtime",
            "ui_text.native_raster_plan.readiness_changed_key_count",
            self.readiness_changes().changed_key_count()
        );
        crate::profile_counter!(
            "runtime",
            "ui_text.native_raster_plan.readiness_full_invalidation_count",
            usize::from(self.readiness_changes().full_invalidation())
        );
        let storage_resource_count = self.storage_resource_count();
        let canonical_frame_plan_count = self.canonical_frame_plan_count();
        let frame_plan_visible_glyph_count = self.submission.gpu_draw.visible_glyph_count;
        let frame_plan_supports_native_submission = self.frame_plan_supports_native_submission();
        let background_composite_replacement_ready = self.background_composite_replacement_ready();
        let mut report = NativeBitmapAtlasPrepareReport {
            frame_index: self.frame_index,
            visible_raster_glyph_count: self.visible_raster_glyph_count,
            source_image_count: self.source_images.len(),
            missing_raster_image_count: self.missing_raster_image_count,
            visible_missing_raster_image_count: self.visible_missing_raster_image_count,
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
            // The sole canonical frame plan owns ordered segments rather than contiguous format
            // runs. These counters describe its resources and visible glyphs directly.
            storage_submission_count: canonical_frame_plan_count,
            storage_submission_visible_glyph_count: frame_plan_visible_glyph_count,
            mixed_storage_replacement_ready: storage_resource_count > 1
                && frame_plan_supports_native_submission,
            requires_background_composite: self.submission.gpu_draw.requires_background_composite,
            background_composite_replacement_ready,
            background_composite_glyph_count: self.background_composite_glyph_count,
            missing_background_composite_glyph_count: self.missing_background_composite_glyph_count,
            source_cache: self.source_cache,
            retry_submission: self.retry_submission,
            retry_state: self.retry_state,
            discarded_stale_retry_glyph_count: self.discarded_stale_retry_glyph_count,
            native_degradation_reason: None,
            first_frame_degradation: None,
            native_submission_ready: self.supports_native_submission(),
            submission: self.submission.submission_report(),
        };
        report.native_degradation_reason =
            native_bitmap_atlas_degradation_reason_for_report(&report);
        report.first_frame_degradation =
            native_bitmap_atlas_first_frame_degradation_for_report(&report);
        report
    }

    fn source_coverage_supports_replacement(&self) -> bool {
        self.visible_raster_glyph_count > 0
            && self.visible_missing_raster_image_count == 0
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

    fn frame_plan_supports_native_submission(&self) -> bool {
        if !self.source_coverage_supports_replacement()
            || !self.background_composite_supports_replacement()
        {
            return false;
        }

        self.source_images.len() == self.visible_raster_glyph_count
            && self.submission.gpu_draw.visible_glyph_count == self.visible_raster_glyph_count
            && self.submission.run.allocation_failures.is_empty()
    }
}

pub(crate) fn native_bitmap_atlas_frame<'a, GlyphRuns>(
    font_database: &FontDatabase,
    raster_worker_pool: Option<&TextRasterWorkerPool>,
    source_cache: &mut NativeBitmapAtlasSourceCache,
    retry_state: &mut GlyphAtlasBitmapRetryFrameState,
    atlas: GlyphAtlasSet,
    viewport_size: UVec2,
    frame_index: u64,
    glyph_runs: GlyphRuns,
) -> NativeBitmapAtlasFrame
where
    GlyphRuns: Clone + IntoIterator<Item = &'a NativeBitmapAtlasGlyphRun>,
{
    crate::profile_scope!(
        "runtime",
        "ui_text.native_raster_plan",
        "native_bitmap_atlas_frame"
    );
    let mut atlas = atlas;
    source_cache.begin_frame();
    let face_epoch = source_cache.face_epoch();
    if let Some(raster_worker_pool) = raster_worker_pool {
        let completion_drain = raster_worker_pool.drain_completed_for_face_epoch(
            face_epoch,
            TextRasterCompletionDrainBudget::new(
                NATIVE_BITMAP_ATLAS_MAX_RASTER_COMPLETIONS_PER_FRAME,
                NATIVE_BITMAP_ATLAS_MAX_RASTER_COMPLETION_BYTES_PER_FRAME,
            ),
        );
        source_cache.apply_worker_completion_drain(completion_drain);
    }
    let budget_evicted_raster_keys = source_cache.take_budget_evicted_raster_keys();
    let invalidated_raster_keys = atlas.invalidate_bitmap_raster_keys(budget_evicted_raster_keys);
    source_cache.invalidate_raster_keys(invalidated_raster_keys);

    let mut source_images = Vec::new();
    let mut visible_raster_glyph_count: usize = 0;
    let mut missing_raster_image_count: usize = 0;
    let mut visible_missing_raster_image_count: usize = 0;
    let mut approximate_raster_image_count: usize = 0;
    let mut unsupported_glyph_count: usize = 0;
    let mut clipped_glyph_count: usize = 0;
    let mut background_composite_glyph_count: usize = 0;
    let mut missing_background_composite_glyph_count: usize = 0;
    let mut pending_placeholders = Vec::new();

    let mut glyph_instance_visit_count = 0usize;
    let mut readiness_by_raster_key = HashMap::new();
    for glyph_run in glyph_runs.clone() {
        for glyph in &glyph_run.glyphs {
            glyph_instance_visit_count = glyph_instance_visit_count.saturating_add(1);
            if readiness_by_raster_key.contains_key(&glyph.raster_key) {
                continue;
            }
            let readiness = match source_cache.cached_image(glyph.raster_key) {
                Some(image) => {
                    let persistent_raster_key =
                        native_bitmap_atlas_raster_key_for_content(glyph.raster_key, image.content);
                    if let Some(raster_key) = persistent_raster_key {
                        let _ =
                            source_cache.bind_persistent_raster_key(glyph.raster_key, raster_key);
                    }
                    NativeBitmapAtlasGlyphReadiness::Exact {
                        image,
                        persistent_raster_key,
                    }
                }
                None => match source_cache.approximate_cached_image(glyph.raster_key) {
                    Some(image) => NativeBitmapAtlasGlyphReadiness::Approximate {
                        image,
                        worker_request: None,
                    },
                    None => NativeBitmapAtlasGlyphReadiness::Missing {
                        worker_request: None,
                    },
                },
            };
            readiness_by_raster_key.insert(glyph.raster_key, readiness);
        }
    }
    crate::profile_counter!(
        "runtime",
        "ui_text.native_raster_plan.glyph_instance_visit_count",
        glyph_instance_visit_count
    );
    crate::profile_counter!(
        "runtime",
        "ui_text.native_raster_plan.unique_raster_dependency_count",
        readiness_by_raster_key.len()
    );

    for glyph_run in glyph_runs {
        for glyph in &glyph_run.glyphs {
            let Some(readiness) = readiness_by_raster_key.get_mut(&glyph.raster_key) else {
                continue;
            };
            let (image, persistent_raster_key, approximate_worker_request) = match readiness {
                NativeBitmapAtlasGlyphReadiness::Exact {
                    image,
                    persistent_raster_key,
                } => (image, *persistent_raster_key, None),
                NativeBitmapAtlasGlyphReadiness::Approximate {
                    image,
                    worker_request,
                } => (image, None, Some(worker_request)),
                NativeBitmapAtlasGlyphReadiness::Missing { worker_request } => {
                    let Some(pending_placeholder) =
                        native_bitmap_atlas_pending_placeholder(glyph, glyph_run, frame_index)
                    else {
                        missing_raster_image_count += 1;
                        continue;
                    };
                    let worker_request = *worker_request.get_or_insert_with(|| {
                        source_cache.request_worker_image(
                            font_database,
                            raster_worker_pool,
                            face_epoch,
                            glyph.raster_key,
                        )
                    });
                    if worker_request_keeps_placeholder(worker_request) {
                        pending_placeholders.push(pending_placeholder);
                    }
                    missing_raster_image_count += 1;
                    visible_missing_raster_image_count += 1;
                    continue;
                }
            };
            let approximate_cache_hit = approximate_worker_request.is_some();
            if image.width == 0 || image.height == 0 {
                continue;
            }
            let screen_rect = native_bitmap_atlas_screen_rect(
                glyph.screen_x,
                glyph.baseline_y,
                image.top,
                image.left,
                image.width,
                image.height,
            );
            let Some(clipped_rect) = screen_rect.clipped_to(glyph_run.bounds) else {
                continue;
            };
            visible_raster_glyph_count += 1;
            if approximate_cache_hit {
                approximate_raster_image_count = approximate_raster_image_count.saturating_add(1);
                if let Some(worker_request) = approximate_worker_request {
                    let _ = worker_request.get_or_insert_with(|| {
                        source_cache.request_worker_image(
                            font_database,
                            raster_worker_pool,
                            face_epoch,
                            glyph.raster_key,
                        )
                    });
                }
            }

            let Some(format) = native_bitmap_atlas_format(image.content) else {
                unsupported_glyph_count += 1;
                continue;
            };
            if native_bitmap_atlas_format_requires_background_composite(format) {
                background_composite_glyph_count += 1;
                if glyph.background_color.is_none() {
                    missing_background_composite_glyph_count += 1;
                }
            }

            let glyph_image = NativeBitmapGlyphImage {
                screen_x: glyph.screen_x,
                baseline_y: glyph.baseline_y,
                top: image.top,
                left: image.left,
                width: image.width,
                height: image.height,
                format,
                source_byte_len: image.bytes.len(),
                foreground_color: native_bitmap_atlas_foreground_color(
                    format,
                    glyph.foreground_color,
                ),
                background_color: native_bitmap_atlas_background_color(
                    format,
                    glyph.background_color,
                ),
            };
            if let Some(clipped_source) = native_bitmap_atlas_source_from_image(
                glyph_image,
                clipped_rect,
                Arc::clone(&image.bytes),
                persistent_raster_key,
            ) {
                clipped_glyph_count += usize::from(clipped_source.was_clipped);
                source_images.push(NativeBitmapAtlasSourceImage {
                    source: clipped_source.source,
                    bytes: clipped_source.bytes,
                    face_epoch: source_cache.face_epoch(),
                });
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
    let invalidated_raster_keys = std::mem::take(&mut submission.run.invalidated_raster_keys);
    source_cache.invalidate_raster_keys(invalidated_raster_keys);
    append_native_bitmap_atlas_pending_placeholders(
        &mut submission,
        source_images.len(),
        pending_placeholders,
        clip_rect,
    );

    let mut source_cache_report = source_cache.frame_report();
    if let Some(raster_worker_pool) = raster_worker_pool {
        source_cache_report.record_worker_pool_diagnostics(raster_worker_pool.diagnostics());
    }

    let readiness_changes = source_cache.take_readiness_changes();
    NativeBitmapAtlasFrame {
        submission,
        source_images,
        frame_index,
        viewport_size,
        clip_rect,
        visible_raster_glyph_count,
        missing_raster_image_count,
        visible_missing_raster_image_count,
        approximate_raster_image_count,
        unsupported_glyph_count,
        clipped_glyph_count,
        background_composite_glyph_count,
        missing_background_composite_glyph_count,
        source_cache: source_cache_report,
        retry_submission: retry_frame.retry_submission,
        retry_state: retry_frame.retry_state,
        discarded_stale_retry_glyph_count: retry_frame.discarded_stale_retry_glyph_count,
        face_epoch: source_cache.face_epoch(),
        readiness_changes,
    }
}

fn worker_request_keeps_placeholder(status: NativeBitmapAtlasWorkerRequestStatus) -> bool {
    matches!(
        status,
        NativeBitmapAtlasWorkerRequestStatus::Submitted(_)
            | NativeBitmapAtlasWorkerRequestStatus::Pending
            | NativeBitmapAtlasWorkerRequestStatus::DeferredByFrameBudget
            | NativeBitmapAtlasWorkerRequestStatus::DeferredByWorkerBackpressure
    )
}

fn native_bitmap_atlas_pending_placeholder(
    glyph: &super::glyph_run::NativeBitmapAtlasGlyph,
    glyph_run: &NativeBitmapAtlasGlyphRun,
    frame_index: u64,
) -> Option<NativeBitmapAtlasPendingPlaceholder> {
    let screen_rect = glyph.placeholder_rect.clipped_to(glyph_run.bounds)?;
    Some(NativeBitmapAtlasPendingPlaceholder {
        format: GlyphAtlasFormat::AlphaMask,
        screen_rect,
        retry_frame_index: frame_index.saturating_add(1),
    })
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

pub(crate) fn bitmap_atlas_page_size() -> UVec2 {
    UVec2::new(512, 512)
}
