use super::super::prepare_report::{
    text_raster_upload_report, ScreenSpaceUiTextSdfGenerationReport,
};
use super::super::resolved_batches::ResolvedScreenSpaceUiTextBatches;
use super::super::*;
use super::support::text_batch;
use crate::text::atlas::GlyphAtlasStorageFormat;
use crate::text::sdf::SdfAtlasRect;

use super::super::super::sdf_atlas::SdfAtlasDirtyPageReport;
use super::super::super::sdf_upload::SdfAtlasUploadPageReport;

#[test]
fn text_prepare_report_summarizes_input_routing_and_sdf_reports() {
    let auto = [text_batch("Auto", UiTextRenderMode::Auto)];
    let native = [text_batch("Native", UiTextRenderMode::Native)];
    let sdf = [text_batch("Sdf", UiTextRenderMode::Sdf)];
    let mut resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&native, &sdf);
    resolved.push_resolved_auto_text(auto[0].clone(), UiTextRenderMode::Sdf);
    let atlas_report = SdfAtlasCacheReport {
        previous_slot_count: 1,
        current_slot_count: 2,
        retained_slot_count: 1,
        stable_slot_count: 1,
        relocated_slot_count: 0,
        added_slot_count: 1,
        evicted_slot_count: 0,
        atlas_resized: false,
        dirty_rect: Some(SdfAtlasRect {
            x: 64,
            y: 0,
            width: 64,
            height: 64,
        }),
        dirty_pages: vec![SdfAtlasDirtyPageReport {
            page_key: crate::text::atlas::GlyphAtlasPageKey::new(
                crate::text::atlas::GlyphAtlasFormat::Sdf,
                0,
            ),
            dirty_rect: SdfAtlasRect {
                x: 64,
                y: 0,
                width: 64,
                height: 64,
            },
        }],
    };
    let mut sdf_report = ScreenSpaceUiSdfPrepareReport {
        text_batch_count: 2,
        atlas_slot_count: 2,
        atlas_size: crate::core::math::UVec2::splat(512),
        atlas_page_count: 1,
        msdf_atlas_page_count: 1,
        atlas_allocation_failure_count: 0,
        atlas_page_limit_failure_count: 0,
        atlas_oversized_failure_count: 0,
        atlas_resized: false,
        bake: Default::default(),
        atlas_upload_byte_len: 512 * 512,
        atlas_upload_full_texture: true,
        atlas_upload: SdfAtlasUploadReport {
            mode: SdfAtlasUploadMode::FullTexture,
            byte_len: 512 * 512,
            full_texture: true,
            dirty_slot_count: 1,
            dirty_rect: Some(SdfAtlasRect {
                x: 64,
                y: 0,
                width: 64,
                height: 64,
            }),
            dirty_byte_len: 4096,
            dirty_pages: vec![SdfAtlasUploadPageReport {
                page_key: crate::text::atlas::GlyphAtlasPageKey::new(
                    crate::text::atlas::GlyphAtlasFormat::Sdf,
                    0,
                ),
                dirty_rect: SdfAtlasRect {
                    x: 64,
                    y: 0,
                    width: 64,
                    height: 64,
                },
                byte_len: 4096,
            }],
        },
        vertex_count: 12,
        vertex_buffer_capacity_byte_len: 4 * 1024,
        vertex_buffer_create_count: 1,
        vertex_buffer_write_byte_len: 720,
        cpu_plan_build_count: 1,
        cpu_plan_reuse_count: 0,
        vertex_plan_build_count: 1,
        vertex_plan_reuse_count: 0,
        decoration_vertex_count: 0,
        material_count: 2,
        draw_count: 2,
        outline_batch_count: 0,
        shadow_batch_count: 0,
        glow_batch_count: 0,
    };
    sdf_report.bake.generation_scheduler.in_flight_batch_count = 3;
    sdf_report
        .bake
        .generation_scheduler
        .completion_backlog_count = 5;
    sdf_report.bake.generation_failure_count = 7;

    let report = text_prepare_report(
        &auto,
        &native,
        &sdf,
        &resolved,
        ScreenSpaceUiTextSdfFallbackReport::default(),
        ScreenSpaceUiNativePrepareReport {
            font_ids: ScreenSpaceUiTextFontIdReport::default(),
            bitmap_atlas: NativeBitmapAtlasPrepareReport {
                frame_index: 0,
                visible_raster_glyph_count: 2,
                source_image_count: 1,
                missing_raster_image_count: 0,
                visible_missing_raster_image_count: 0,
                approximate_raster_image_count: 0,
                unsupported_glyph_count: 1,
                clipped_glyph_count: 0,
                atlas_storage_format: Some(GlyphAtlasStorageFormat::R8Unorm),
                mixed_atlas_storage_format: false,
                storage_submission_count: 1,
                storage_submission_visible_glyph_count: 1,
                mixed_storage_replacement_ready: false,
                requires_background_composite: false,
                background_composite_replacement_ready: false,
                background_composite_glyph_count: 0,
                missing_background_composite_glyph_count: 0,
                source_cache: Default::default(),
                retry_submission: Default::default(),
                retry_state: Default::default(),
                discarded_stale_retry_glyph_count: 0,
                glyphon_fallback_reason: Some(
                    native_bitmap_atlas::NativeBitmapAtlasGlyphonFallbackReason::UnsupportedGlyphFormat,
                ),
                first_frame_degradation: None,
                replaces_glyphon: false,
                submission: Default::default(),
            },
        },
        MissingGlyphDiagnosticsReport::default(),
        GlyphAtlasBitmapRendererPrepareReport::default(),
        atlas_report.clone(),
        sdf_report.clone(),
    );

    assert_eq!(
        report,
        ScreenSpaceUiTextPrepareReport {
            input_auto_text_batch_count: 1,
            input_native_text_batch_count: 1,
            input_sdf_text_batch_count: 1,
            resolved_native_text_batch_count: 1,
            resolved_sdf_text_batch_count: 2,
            auto_route: Default::default(),
            sdf_fallback: ScreenSpaceUiTextSdfFallbackReport::default(),
            native_font_ids: ScreenSpaceUiTextFontIdReport::default(),
            missing_glyphs: MissingGlyphDiagnosticsReport::default(),
            layout_fallbacks: crate::text::TextLayoutFallbackReport::default(),
            raster_upload: ScreenSpaceUiTextRasterUploadReport {
                visible_raster_glyph_count: 2,
                source_image_count: 1,
                ..ScreenSpaceUiTextRasterUploadReport::default()
            },
            native_bitmap_atlas: NativeBitmapAtlasPrepareReport {
                frame_index: 0,
                visible_raster_glyph_count: 2,
                source_image_count: 1,
                missing_raster_image_count: 0,
                visible_missing_raster_image_count: 0,
                approximate_raster_image_count: 0,
                unsupported_glyph_count: 1,
                clipped_glyph_count: 0,
                atlas_storage_format: Some(GlyphAtlasStorageFormat::R8Unorm),
                mixed_atlas_storage_format: false,
                storage_submission_count: 1,
                storage_submission_visible_glyph_count: 1,
                mixed_storage_replacement_ready: false,
                requires_background_composite: false,
                background_composite_replacement_ready: false,
                background_composite_glyph_count: 0,
                missing_background_composite_glyph_count: 0,
                source_cache: Default::default(),
                retry_submission: Default::default(),
                retry_state: Default::default(),
                discarded_stale_retry_glyph_count: 0,
                glyphon_fallback_reason: Some(
                    native_bitmap_atlas::NativeBitmapAtlasGlyphonFallbackReason::UnsupportedGlyphFormat,
                ),
                first_frame_degradation: None,
                replaces_glyphon: false,
                submission: Default::default(),
            },
            bitmap_atlas_renderer: GlyphAtlasBitmapRendererPrepareReport::default(),
            sdf_atlas: atlas_report,
            sdf_generation: ScreenSpaceUiTextSdfGenerationReport {
                pending_batch_count: 3,
                completion_backlog_count: 5,
                failure_count: 7,
            },
            sdf_renderer: sdf_report,
        }
    );
}

#[test]
fn text_prepare_report_exposes_all_outstanding_raster_work() {
    let native = [text_batch("Native", UiTextRenderMode::Native)];
    let resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&native, &[]);
    let native_bitmap_atlas = NativeBitmapAtlasPrepareReport {
        frame_index: 7,
        visible_raster_glyph_count: 5,
        source_image_count: 4,
        missing_raster_image_count: 1,
        approximate_raster_image_count: 2,
        source_cache: native_bitmap_atlas::NativeBitmapAtlasSourceCacheFrameReport {
            hit_count: 6,
            approximate_hit_count: 2,
            approximate_probe_count: 5,
            miss_count: 3,
            insert_count: 4,
            capacity: 8,
            entry_count: 6,
            persistent_raster_key_count: 5,
            resident_byte_count: 4096,
            max_byte_count: 8192,
            lru_repair_count: 7,
            lru_touch_count: 7,
            evicted_count: 9,
            evicted_byte_count: 2048,
            budget_linked_eviction_count: 2,
            linked_raster_invalidation_count: 3,
            rejected_byte_budget_count: 11,
            invalidated_count: 13,
            worker_request_submitted_count: 2,
            worker_request_pending_count: 1,
            pending_worker_count: 3,
            worker_request_deferred_count: 5,
            worker_request_unavailable_count: 1,
            worker_request_backpressured_count: 8,
            worker_request_font_copied_byte_count: 1024,
            worker_raster_font_resident_byte_count: 6144,
            worker_raster_font_entry_count: 2,
            worker_request_cancelled_count: 13,
            worker_request_failed_count: 2,
            worker_completion_failed_count: 3,
            worker_completion_invalid_bitmap_count: 4,
            worker_completion_applied_byte_count: 2048,
            worker_completion_drained_byte_count: 4096,
            worker_completion_byte_budget_deferred_count: 1,
            worker_completion_oversized_accepted_count: 1,
            worker_pool_budgeted_thread_count: 2,
            worker_pool_in_flight_count: 17,
            worker_pool_queued_count: 11,
            worker_pool_queued_input_byte_count: 3072,
            worker_pool_running_count: 6,
            worker_pool_completed_total: 19,
            worker_pool_failed_total: 23,
            worker_pool_queue_peak_count: 29,
            worker_pool_completion_backlog_count: 7,
            worker_pool_completion_backlog_byte_count: 5120,
            worker_pool_completion_backpressured_total: 31,
            worker_pool_completion_budget_rejected_total: 37,
            worker_pool_completion_rejected_byte_total: 6144,
            worker_pool_request_backpressured_total: 41,
            worker_pool_cancelled_total: 43,
            ..Default::default()
        },
        submission: crate::text::atlas::GlyphAtlasBitmapRenderSubmissionReport {
            visible_placeholder_count: 1,
            upload_command_count: 3,
            upload_copy_count: 3,
            upload_byte_len: 384,
            resident_page_byte_len: 16_384,
            ..Default::default()
        },
        ..NativeBitmapAtlasPrepareReport::default()
    };
    let bitmap_renderer = GlyphAtlasBitmapRendererPrepareReport::default()
        .with_upload_counters_for_test(3, 384, 1, 1, false);

    let report = text_prepare_report(
        &[],
        &native,
        &[],
        &resolved,
        ScreenSpaceUiTextSdfFallbackReport::default(),
        ScreenSpaceUiNativePrepareReport {
            font_ids: ScreenSpaceUiTextFontIdReport::default(),
            bitmap_atlas: native_bitmap_atlas,
        },
        MissingGlyphDiagnosticsReport::default(),
        bitmap_renderer,
        SdfAtlasCacheReport::default(),
        ScreenSpaceUiSdfPrepareReport::default(),
    );

    assert_eq!(
        report.raster_upload,
        ScreenSpaceUiTextRasterUploadReport {
            visible_raster_glyph_count: 5,
            source_image_count: 4,
            missing_raster_image_count: 1,
            visible_missing_raster_image_count: 0,
            visible_placeholder_count: 1,
            approximate_raster_image_count: 2,
            source_cache_hit_count: 6,
            source_cache_approximate_hit_count: 2,
            source_cache_miss_count: 3,
            source_cache_insert_count: 4,
            source_cache_capacity: 8,
            source_cache_entry_count: 6,
            source_cache_persistent_raster_key_count: 5,
            source_cache_resident_byte_count: 4096,
            source_cache_max_byte_count: 8192,
            source_cache_approximate_probe_count: 5,
            source_cache_lru_repair_count: 7,
            source_cache_lru_touch_count: 7,
            source_cache_evicted_count: 9,
            source_cache_evicted_byte_count: 2048,
            source_cache_budget_linked_eviction_count: 2,
            source_cache_linked_raster_invalidation_count: 3,
            source_cache_rejected_byte_budget_count: 11,
            source_cache_invalidated_count: 13,
            atlas_slot_cache_hit_count: 0,
            atlas_slot_cache_miss_count: 0,
            atlas_slot_cache_insert_count: 0,
            atlas_resident_page_byte_len: 16_384,
            worker_request_submitted_count: 2,
            worker_pending_count: 3,
            worker_request_deferred_count: 5,
            worker_request_unavailable_count: 1,
            worker_request_backpressured_count: 8,
            worker_request_font_copied_byte_count: 1024,
            worker_raster_font_resident_byte_count: 6144,
            worker_raster_font_entry_count: 2,
            worker_request_cancelled_count: 13,
            worker_completion_applied_byte_count: 2048,
            worker_completion_drained_byte_count: 4096,
            worker_completion_byte_budget_deferred_count: 1,
            worker_completion_oversized_accepted_count: 1,
            worker_pool_budgeted_thread_count: 2,
            worker_pool_in_flight_count: 17,
            worker_pool_queued_count: 11,
            worker_pool_queued_input_byte_count: 3072,
            worker_pool_running_count: 6,
            worker_pool_completed_total: 19,
            worker_pool_failed_total: 23,
            worker_pool_queue_peak_count: 29,
            worker_pool_completion_backlog_count: 7,
            worker_pool_completion_backlog_byte_count: 5120,
            worker_pool_completion_backpressured_total: 31,
            worker_pool_completion_budget_rejected_total: 37,
            worker_pool_completion_rejected_byte_total: 6144,
            worker_pool_request_backpressured_total: 41,
            worker_pool_cancelled_total: 43,
            worker_failed_count: 9,
            upload_command_count: 3,
            upload_copy_count: 3,
            upload_copy_byte_len: 0,
            upload_byte_len: 384,
            renderer_upload_request_count: 3,
            renderer_upload_byte_len: 384,
            renderer_upload_requeued_count: 1,
            renderer_upload_failure_count: 1,
            renderer_upload_ready_to_write_texture: false,
        }
    );
}

#[test]
fn text_raster_upload_report_separates_offscreen_missing_images() {
    let report = text_raster_upload_report(
        &NativeBitmapAtlasPrepareReport {
            missing_raster_image_count: 1,
            visible_missing_raster_image_count: 0,
            ..NativeBitmapAtlasPrepareReport::default()
        },
        &GlyphAtlasBitmapRendererPrepareReport::default(),
    );

    assert_eq!(report.missing_raster_image_count, 1);
    assert_eq!(report.visible_missing_raster_image_count, 0);
}

#[test]
fn text_prepare_report_exposes_raster_upload_scroll_counters() {
    let report = text_raster_upload_report(
        &NativeBitmapAtlasPrepareReport {
            source_cache: native_bitmap_atlas::NativeBitmapAtlasSourceCacheFrameReport {
                hit_count: 2,
                miss_count: 3,
                insert_count: 3,
                ..Default::default()
            },
            submission: crate::text::atlas::GlyphAtlasBitmapRenderSubmissionReport {
                slot_cache_hit_count: 2,
                slot_cache_miss_count: 3,
                slot_cache_insert_count: 3,
                upload_copy_count: 3,
                upload_copy_byte_len: 192,
                ..Default::default()
            },
            ..NativeBitmapAtlasPrepareReport::default()
        },
        &GlyphAtlasBitmapRendererPrepareReport::default(),
    );

    assert_eq!(report.source_cache_hit_count, 2);
    assert_eq!(report.source_cache_miss_count, 3);
    assert_eq!(report.source_cache_insert_count, 3);
    assert_eq!(report.atlas_slot_cache_hit_count, 2);
    assert_eq!(report.atlas_slot_cache_miss_count, 3);
    assert_eq!(report.atlas_slot_cache_insert_count, 3);
    assert_eq!(report.upload_copy_count, 3);
    assert_eq!(report.upload_copy_byte_len, 192);
}

#[test]
fn raster_upload_report_excludes_planned_placeholders_when_glyphon_fallback_wins() {
    let native_bitmap_atlas = NativeBitmapAtlasPrepareReport {
        visible_raster_glyph_count: 2,
        source_image_count: 1,
        storage_submission_visible_glyph_count: 1,
        submission: crate::text::atlas::GlyphAtlasBitmapRenderSubmissionReport {
            visible_placeholder_count: 1,
            ..Default::default()
        },
        retry_submission: crate::text::atlas::GlyphAtlasBitmapRetryFrameSubmissionReport {
            rejected_new_source_count: 1,
            rejected_new_source_byte_count: 1024 * 1024 + 1,
            ..Default::default()
        },
        ..NativeBitmapAtlasPrepareReport::default()
    };

    let report = text_raster_upload_report(
        &native_bitmap_atlas,
        &GlyphAtlasBitmapRendererPrepareReport::default(),
    );

    assert_eq!(report.visible_placeholder_count, 0);
}

#[test]
fn profiling_recorder_visibility_reaches_the_text_owner() {
    let source = include_str!("../prepare_report/profile.rs");

    assert!(source.contains("pub(in super::super) fn record_text_prepare_profile"));
    assert!(!source.contains("pub(super) fn record_text_prepare_profile"));
}
