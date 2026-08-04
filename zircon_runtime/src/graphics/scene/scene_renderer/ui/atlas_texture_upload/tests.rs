use crate::core::math::UVec2;
use crate::text::atlas::{
    GlyphAtlasBitmapFaceValidity, GlyphAtlasBitmapPageUploadStaging,
    GlyphAtlasBitmapPreparedUploadPlan, GlyphAtlasBitmapSource, GlyphAtlasBitmapStagedUpload,
    GlyphAtlasBitmapStagedUploadFailure, GlyphAtlasBitmapStagedUploadFailureReason,
    GlyphAtlasBitmapStagedUploadPlan, GlyphAtlasBitmapTextureUploadRequest,
    GlyphAtlasBitmapUploadSourceBytes, GlyphAtlasBitmapUploadStagingFailure,
    GlyphAtlasBitmapUploadStagingFailureReason, GlyphAtlasBitmapUploadStagingPlan,
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasRect,
    GlyphAtlasSamplingSemantics, GlyphAtlasSet, GlyphAtlasStorageFormat, GlyphAtlasUploadCommand,
    GlyphAtlasUploadMode, glyph_atlas_bitmap_render_submission_plan_with_padding,
    render_plan::GlyphAtlasScreenRect,
};

use super::binding::{
    GlyphAtlasBitmapTextureUploadBindingFailureReason,
    glyph_atlas_bitmap_texture_upload_binding_plan,
};
use super::resource::glyph_atlas_texture_array_spec;
use super::write::glyph_atlas_texture_upload_write;
use super::{
    GlyphAtlasBitmapTextureUploadFramePlan, GlyphAtlasBitmapTextureUploadFrameReport,
    GlyphAtlasTextureArrayResources,
    glyph_atlas_bitmap_render_submission_texture_upload_frame_report,
    glyph_atlas_bitmap_texture_upload_frame_plan,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas,
    glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity,
    write_glyph_atlas_bitmap_texture_upload_frame_resources,
};

#[test]
fn glyph_atlas_texture_array_spec_maps_r8_storage_to_2d_array_resource() {
    let spec = glyph_atlas_texture_array_spec(
        "test-alpha-atlas",
        "test-alpha-atlas-view",
        GlyphAtlasFormat::AlphaMask.storage_format(),
        UVec2::new(256, 128),
        3,
    );

    assert!(spec.matches_storage(GlyphAtlasStorageFormat::R8Unorm));
    assert_eq!(spec.texture_label, "test-alpha-atlas");
    assert_eq!(spec.view_label, "test-alpha-atlas-view");
    assert_eq!(spec.format, wgpu::TextureFormat::R8Unorm);
    assert_eq!(spec.extent().width, 256);
    assert_eq!(spec.extent().height, 128);
    assert_eq!(spec.extent().depth_or_array_layers, 3);
    assert!(spec.usage.contains(wgpu::TextureUsages::COPY_DST));
    assert!(spec.usage.contains(wgpu::TextureUsages::TEXTURE_BINDING));
}

#[test]
fn glyph_atlas_texture_array_spec_maps_rgba_bitmap_storage_to_2d_array_resource() {
    let subpixel_spec = glyph_atlas_texture_array_spec(
        "test-subpixel-atlas",
        "test-subpixel-atlas-view",
        GlyphAtlasFormat::SubpixelMask.storage_format(),
        UVec2::new(0, 0),
        0,
    );
    let color_spec = glyph_atlas_texture_array_spec(
        "test-color-atlas",
        "test-color-atlas-view",
        GlyphAtlasFormat::Color.storage_format(),
        UVec2::new(64, 32),
        2,
    );

    assert!(subpixel_spec.matches_storage(GlyphAtlasStorageFormat::Rgba8Unorm));
    assert_eq!(subpixel_spec.format, wgpu::TextureFormat::Rgba8Unorm);
    assert_eq!(subpixel_spec.extent().width, 1);
    assert_eq!(subpixel_spec.extent().height, 1);
    assert_eq!(subpixel_spec.extent().depth_or_array_layers, 1);
    assert_eq!(color_spec.format, wgpu::TextureFormat::Rgba8Unorm);
    assert_eq!(color_spec.extent().width, 64);
    assert_eq!(color_spec.extent().height, 32);
    assert_eq!(color_spec.extent().depth_or_array_layers, 2);
}

#[test]
fn glyph_atlas_texture_upload_write_projects_command_to_wgpu_fields() {
    let command = GlyphAtlasUploadCommand {
        mode: GlyphAtlasUploadMode::PartialRect,
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 3),
        page_generation: 5,
        sampling_semantics: GlyphAtlasSamplingSemantics::AlphaCoverage,
        rect: GlyphAtlasRect {
            x: 7,
            y: 11,
            width: 19,
            height: 23,
        },
        source_offset: 4096,
        bytes_per_row: 256,
        rows_per_image: 64,
        upload_byte_len: 19 * 23,
    };

    let write = glyph_atlas_texture_upload_write(command);

    assert_eq!(write.origin_x, 7);
    assert_eq!(write.origin_y, 11);
    assert_eq!(write.origin_layer, 3);
    assert_eq!(write.source_offset, 4096);
    assert_eq!(write.bytes_per_row, 256);
    assert_eq!(write.rows_per_image, 64);
    assert_eq!(write.extent_width, 19);
    assert_eq!(write.extent_height, 23);
    assert_eq!(write.extent_layers, 1);
}

#[test]
fn bitmap_texture_upload_binding_plan_binds_request_to_staging_page_bytes() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let staging_pages = vec![bitmap_staging_page(page_key, 16, 64)];
    let request = bitmap_upload_request(page_key);

    let plan = glyph_atlas_bitmap_texture_upload_binding_plan(&staging_pages, &[request]);

    assert!(plan.has_bindings());
    assert!(!plan.has_failures());
    assert_eq!(plan.bindings.len(), 1);
    let binding = plan.bindings[0];
    assert_eq!(binding.request_index, 0);
    assert_eq!(binding.bytes, staging_pages[0].bytes.as_slice());
    assert_eq!(binding.write.origin_x, 3);
    assert_eq!(binding.write.origin_y, 5);
    assert_eq!(binding.write.origin_layer, 2);
    assert_eq!(binding.write.source_offset, 18);
    assert_eq!(binding.write.bytes_per_row, 16);
    assert_eq!(binding.write.rows_per_image, 4);
    assert_eq!(binding.write.extent_width, 4);
    assert_eq!(binding.write.extent_height, 2);
    assert_eq!(binding.write.extent_layers, 1);
}

#[test]
fn bitmap_texture_upload_binding_plan_reports_missing_staging_page() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let mut request = bitmap_upload_request(page_key);
    request.staging_page_index = 3;

    let plan = glyph_atlas_bitmap_texture_upload_binding_plan(&[], &[request]);

    assert!(!plan.has_bindings());
    assert_eq!(
        plan.failures[0].reason,
        GlyphAtlasBitmapTextureUploadBindingFailureReason::MissingStagingPage
    );
}

#[test]
fn bitmap_texture_upload_binding_plan_reports_staging_page_key_mismatch() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let other_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 3);
    let staging_pages = vec![bitmap_staging_page(other_key, 16, 64)];

    let plan = glyph_atlas_bitmap_texture_upload_binding_plan(
        &staging_pages,
        &[bitmap_upload_request(page_key)],
    );

    assert!(!plan.has_bindings());
    assert_eq!(
        plan.failures[0].reason,
        GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageKeyMismatch
    );
}

#[test]
fn bitmap_texture_upload_binding_plan_reports_staging_page_generation_mismatch() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let staging_pages = vec![bitmap_staging_page_with_generation(page_key, 0, 16, 64)];
    let mut request = bitmap_upload_request(page_key);
    request.page_generation = 1;

    let plan = glyph_atlas_bitmap_texture_upload_binding_plan(&staging_pages, &[request]);

    assert!(!plan.has_bindings());
    assert_eq!(
        plan.failures[0].reason,
        GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageGenerationMismatch
    );
}

#[test]
fn bitmap_texture_upload_binding_plan_reports_staging_page_target_rect_mismatch() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let staging_pages = vec![bitmap_staging_page(page_key, 16, 64)];
    let mut request = bitmap_upload_request(page_key);
    request.extent = UVec2::new(3, 2);

    let plan = glyph_atlas_bitmap_texture_upload_binding_plan(&staging_pages, &[request]);

    assert!(!plan.has_bindings());
    assert_eq!(
        plan.failures[0].reason,
        GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageTargetRectMismatch
    );
}

#[test]
fn bitmap_texture_upload_binding_plan_reports_staging_page_stride_mismatch() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let staging_pages = vec![bitmap_staging_page(page_key, 24, 64)];

    let plan = glyph_atlas_bitmap_texture_upload_binding_plan(
        &staging_pages,
        &[bitmap_upload_request(page_key)],
    );

    assert!(!plan.has_bindings());
    assert_eq!(
        plan.failures[0].reason,
        GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageRowStrideMismatch
    );
}

#[test]
fn bitmap_texture_upload_binding_plan_reports_staging_page_length_mismatch() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let staging_pages = vec![bitmap_staging_page(page_key, 16, 60)];

    let plan = glyph_atlas_bitmap_texture_upload_binding_plan(
        &staging_pages,
        &[bitmap_upload_request(page_key)],
    );

    assert!(!plan.has_bindings());
    assert_eq!(
        plan.failures[0].reason,
        GlyphAtlasBitmapTextureUploadBindingFailureReason::StagingPageByteLengthMismatch
    );
}

#[test]
fn bitmap_texture_upload_binding_plan_reports_request_range_out_of_bounds() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let staging_pages = vec![bitmap_staging_page(page_key, 16, 64)];
    let mut request = bitmap_upload_request(page_key);
    request.source_offset = 58;

    let plan = glyph_atlas_bitmap_texture_upload_binding_plan(&staging_pages, &[request]);

    assert!(!plan.has_bindings());
    assert_eq!(
        plan.failures[0].reason,
        GlyphAtlasBitmapTextureUploadBindingFailureReason::RequestRangeOutOfBounds
    );
}

#[test]
fn bitmap_texture_upload_frame_plan_binds_prepared_uploads_for_write() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let prepared = prepared_upload_plan(
        vec![bitmap_staging_page(page_key, 16, 64)],
        vec![bitmap_staged_upload(page_key)],
        Vec::new(),
        Vec::new(),
    );

    let plan = glyph_atlas_bitmap_texture_upload_frame_plan(&prepared);
    let report = plan.report();

    assert!(plan.ready_to_write_texture());
    assert_eq!(report.request_count, 1);
    assert_eq!(report.binding_count, 1);
    assert_eq!(report.upload_byte_len, 8);
    assert!(!report.has_failures());
}

#[test]
fn bitmap_texture_upload_frame_plan_blocks_write_on_binding_failure() {
    let request_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let staging_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 3);
    let prepared = prepared_upload_plan(
        vec![bitmap_staging_page(staging_key, 16, 64)],
        vec![bitmap_staged_upload(request_key)],
        Vec::new(),
        Vec::new(),
    );

    let plan = glyph_atlas_bitmap_texture_upload_frame_plan(&prepared);
    let report = plan.report();

    assert!(!plan.ready_to_write_texture());
    assert_eq!(report.request_count, 1);
    assert_eq!(report.binding_count, 0);
    assert_eq!(report.binding_failure_count, 1);
    assert_eq!(report.upload_byte_len, 0);
    assert!(report.has_failures());
}

#[test]
fn bitmap_texture_upload_frame_plan_reports_staging_and_staged_failures() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let prepared = prepared_upload_plan(
        vec![bitmap_staging_page(page_key, 16, 64)],
        Vec::new(),
        vec![GlyphAtlasBitmapUploadStagingFailure {
            source_index: 7,
            page_key,
            reason: GlyphAtlasBitmapUploadStagingFailureReason::MissingSourceBytes,
        }],
        vec![GlyphAtlasBitmapStagedUploadFailure {
            upload_command_index: 3,
            page_key,
            reason: GlyphAtlasBitmapStagedUploadFailureReason::MissingStagingPage,
        }],
    );

    let plan = glyph_atlas_bitmap_texture_upload_frame_plan(&prepared);
    let report = plan.report();

    assert!(!plan.ready_to_write_texture());
    assert_eq!(report.request_count, 0);
    assert_eq!(report.binding_count, 0);
    assert_eq!(report.staging_failure_count, 1);
    assert_eq!(report.staged_upload_failure_count, 1);
    assert_eq!(report.skipped_staged_upload_failure_count, 1);
    assert!(report.has_failures());
}

#[test]
fn bitmap_texture_upload_frame_plan_discards_stale_page_generation() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let prepared = prepared_upload_plan(
        vec![bitmap_staging_page_with_generation(page_key, 0, 16, 64)],
        vec![bitmap_staged_upload_with_generation(page_key, 0)],
        Vec::new(),
        Vec::new(),
    );
    let current_atlas = GlyphAtlasSet::from_page(
        GlyphAtlasPageSpec::new(page_key, UVec2::new(8, 4)).with_generation(1),
    );

    let plan = glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas(&prepared, &current_atlas);
    let report = plan.report();

    assert!(!plan.ready_to_write_texture());
    assert_eq!(report.request_count, 0);
    assert_eq!(report.binding_count, 0);
    assert_eq!(report.requeued_upload_count, 1);
    assert_eq!(report.page_generation_mismatch_requeue_count, 1);
    assert_eq!(report.missing_page_requeue_count, 0);
    assert_eq!(report.stale_page_generation_count, 1);
    assert_eq!(report.face_invalidated_count, 0);
    assert_eq!(report.upload_byte_len, 0);
    assert!(report.has_failures());
}

#[test]
fn bitmap_texture_upload_frame_plan_reports_missing_page_requeue() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let prepared = prepared_upload_plan(
        vec![bitmap_staging_page_with_generation(page_key, 0, 16, 64)],
        vec![bitmap_staged_upload_with_generation(page_key, 0)],
        Vec::new(),
        Vec::new(),
    );
    let empty_atlas = GlyphAtlasSet::default();

    let plan = glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas(&prepared, &empty_atlas);
    let report = plan.report();

    assert!(!plan.ready_to_write_texture());
    assert_eq!(report.request_count, 0);
    assert_eq!(report.binding_count, 0);
    assert_eq!(report.requeued_upload_count, 1);
    assert_eq!(report.missing_page_requeue_count, 1);
    assert_eq!(report.page_generation_mismatch_requeue_count, 0);
    assert_eq!(report.stale_page_generation_count, 1);
    assert_eq!(report.face_invalidated_count, 0);
    assert_eq!(report.upload_byte_len, 0);
    assert!(report.has_failures());
}

#[test]
fn bitmap_texture_upload_frame_plan_reports_face_invalidated_requeue() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 2);
    let prepared = prepared_upload_plan(
        vec![bitmap_staging_page_with_generation(page_key, 0, 16, 64)],
        vec![bitmap_staged_upload_with_generation(page_key, 0)],
        Vec::new(),
        Vec::new(),
    );
    let current_atlas = GlyphAtlasSet::from_page(
        GlyphAtlasPageSpec::new(page_key, UVec2::new(8, 4)).with_generation(0),
    );

    let plan = glyph_atlas_bitmap_texture_upload_frame_plan_for_atlas_and_face_validity(
        &prepared,
        &current_atlas,
        GlyphAtlasBitmapFaceValidity::Invalidated,
    );
    let report = plan.report();

    assert!(!plan.ready_to_write_texture());
    assert_eq!(report.request_count, 0);
    assert_eq!(report.binding_count, 0);
    assert_eq!(report.requeued_upload_count, 1);
    assert_eq!(report.missing_page_requeue_count, 0);
    assert_eq!(report.page_generation_mismatch_requeue_count, 0);
    assert_eq!(report.stale_page_generation_count, 0);
    assert_eq!(report.face_invalidated_count, 1);
    assert_eq!(report.upload_byte_len, 0);
    assert!(report.has_failures());
}

#[test]
fn bitmap_texture_upload_frame_resources_writer_targets_texture_array_resources() {
    let _writer: for<'a> fn(
        &wgpu::Queue,
        &GlyphAtlasTextureArrayResources,
        &GlyphAtlasBitmapTextureUploadFramePlan<'a>,
    ) -> GlyphAtlasBitmapTextureUploadFrameReport =
        write_glyph_atlas_bitmap_texture_upload_frame_resources;
}

#[test]
fn bitmap_texture_upload_frame_report_prepares_render_submission_source_bytes() {
    let submission = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [bitmap_source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 4),
            8.0,
            32,
        )],
        UVec2::new(32, 32),
        79,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );
    let source_bytes = vec![13; 32];

    let report = glyph_atlas_bitmap_render_submission_texture_upload_frame_report(
        &submission,
        [GlyphAtlasBitmapUploadSourceBytes::new(
            0,
            source_bytes.as_slice(),
        )],
    );

    assert_eq!(report.request_count, 1);
    assert_eq!(report.binding_count, 1);
    assert_eq!(report.upload_byte_len, 32);
    assert!(report.ready_to_write_texture);
    assert!(!report.has_failures());
}

#[test]
fn bitmap_texture_upload_frame_report_blocks_missing_submission_source_bytes() {
    let submission = glyph_atlas_bitmap_render_submission_plan_with_padding(
        [bitmap_source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(8, 4),
            8.0,
            32,
        )],
        UVec2::new(32, 32),
        83,
        1,
        2,
        UVec2::new(80, 32),
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    let report = glyph_atlas_bitmap_render_submission_texture_upload_frame_report(
        &submission,
        std::iter::empty::<GlyphAtlasBitmapUploadSourceBytes<'_>>(),
    );

    assert_eq!(report.request_count, 0);
    assert_eq!(report.binding_count, 0);
    assert_eq!(report.staging_failure_count, 1);
    assert!(!report.ready_to_write_texture);
    assert!(report.has_failures());
}

fn bitmap_staging_page(
    page_key: GlyphAtlasPageKey,
    bytes_per_row: u32,
    byte_len: usize,
) -> GlyphAtlasBitmapPageUploadStaging {
    bitmap_staging_page_with_generation(page_key, 0, bytes_per_row, byte_len)
}

fn bitmap_staging_page_with_generation(
    page_key: GlyphAtlasPageKey,
    page_generation: u64,
    bytes_per_row: u32,
    byte_len: usize,
) -> GlyphAtlasBitmapPageUploadStaging {
    GlyphAtlasBitmapPageUploadStaging {
        page_key,
        page_generation,
        target_rect: GlyphAtlasRect {
            x: 3,
            y: 5,
            width: 4,
            height: 2,
        },
        bytes_per_row,
        bytes: (0..byte_len).map(|value| value as u8).collect(),
    }
}

fn prepared_upload_plan(
    pages: Vec<GlyphAtlasBitmapPageUploadStaging>,
    uploads: Vec<GlyphAtlasBitmapStagedUpload>,
    staging_failures: Vec<GlyphAtlasBitmapUploadStagingFailure>,
    staged_failures: Vec<GlyphAtlasBitmapStagedUploadFailure>,
) -> GlyphAtlasBitmapPreparedUploadPlan {
    GlyphAtlasBitmapPreparedUploadPlan {
        staging: GlyphAtlasBitmapUploadStagingPlan {
            pages,
            failures: staging_failures,
        },
        staged_uploads: GlyphAtlasBitmapStagedUploadPlan {
            uploads,
            failures: staged_failures,
        },
    }
}

fn bitmap_staged_upload(page_key: GlyphAtlasPageKey) -> GlyphAtlasBitmapStagedUpload {
    bitmap_staged_upload_with_generation(page_key, 0)
}

fn bitmap_staged_upload_with_generation(
    page_key: GlyphAtlasPageKey,
    page_generation: u64,
) -> GlyphAtlasBitmapStagedUpload {
    GlyphAtlasBitmapStagedUpload {
        staging_page_index: 0,
        command: bitmap_upload_command_with_generation(page_key, page_generation),
        staging_page_byte_len: 64,
    }
}

fn bitmap_upload_command(page_key: GlyphAtlasPageKey) -> GlyphAtlasUploadCommand {
    bitmap_upload_command_with_generation(page_key, 0)
}

fn bitmap_upload_command_with_generation(
    page_key: GlyphAtlasPageKey,
    page_generation: u64,
) -> GlyphAtlasUploadCommand {
    GlyphAtlasUploadCommand {
        mode: GlyphAtlasUploadMode::PartialRect,
        page_key,
        page_generation,
        sampling_semantics: GlyphAtlasSamplingSemantics::AlphaCoverage,
        rect: GlyphAtlasRect {
            x: 3,
            y: 5,
            width: 4,
            height: 2,
        },
        source_offset: 18,
        bytes_per_row: 16,
        rows_per_image: 4,
        upload_byte_len: 8,
    }
}

fn bitmap_upload_request(page_key: GlyphAtlasPageKey) -> GlyphAtlasBitmapTextureUploadRequest {
    GlyphAtlasBitmapTextureUploadRequest {
        staging_page_index: 0,
        page_key,
        page_generation: 0,
        origin_xy: UVec2::new(3, 5),
        origin_layer: page_key.page_index,
        extent: UVec2::new(4, 2),
        source_offset: 18,
        bytes_per_row: 16,
        rows_per_image: 4,
        upload_byte_len: 8,
        staging_page_byte_len: 64,
    }
}

fn bitmap_source(
    format: GlyphAtlasFormat,
    content_size: UVec2,
    x: f32,
    source_byte_len: usize,
) -> GlyphAtlasBitmapSource {
    GlyphAtlasBitmapSource {
        raster_key: None,
        format,
        content_size,
        screen_rect: GlyphAtlasScreenRect::new(
            x,
            6.0,
            content_size.x as f32,
            content_size.y as f32,
        ),
        foreground_color: [0.86, 0.88, 0.9, 1.0],
        background_color: [0.08, 0.09, 0.1, 1.0],
        source_byte_len,
    }
}
