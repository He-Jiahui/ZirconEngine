use super::*;

#[test]
fn sdf_renderer_uses_persistent_capacity_managed_vertex_buffer() {
    let renderer = include_str!("../../sdf_render.rs");
    let text_system = include_str!("../../text.rs");
    let cpu_frame = include_str!("../../text/sdf_cpu_frame.rs");
    let buffer = include_str!("../vertex_buffer.rs");

    assert!(!renderer.contains("create_buffer_init"));
    assert!(renderer.contains("write_sdf_vertex_buffer("));
    assert!(renderer.contains("vertices: Vec<ScreenSpaceUiSdfVertex>"));
    assert!(renderer.contains("self.vertices.clear();"));
    assert!(renderer.contains("self.draw_plan.rebuild("));
    assert!(renderer.contains("self.compiled_frame.matches("));
    assert!(!renderer.contains("let mut vertices = build_text_decoration_vertices"));
    assert!(!renderer.contains("text_state.build_sdf_atlas"));
    assert!(!renderer.contains("generation_failures_for_plan"));
    assert!(!renderer.contains("prepare_text_runs_cpu_for_frame"));
    assert!(renderer.contains("atlas_bake: &SdfAtlasBake"));
    assert!(text_system.contains("let mut sdf_atlas_bake = self.text_state.build_sdf_atlas("));
    assert!(
        text_system.contains("record_generation_failures(&sdf_atlas_bake.generation_failures)")
    );
    assert!(text_system.contains("self.sdf_cpu_frame.prepare("));
    assert!(text_system.contains("self.sdf_cpu_frame.invalidate();"));
    assert!(cpu_frame.contains("prepared_sdf_texts"));
    assert!(cpu_frame.contains("prepared_native_texts"));
    assert!(cpu_frame.contains("text_cpu_inputs_match"));
    assert!(buffer.contains("wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST"));
    assert!(buffer.contains("requires_reallocation"));
    assert!(buffer.contains("payload_changed"));
    assert!(buffer.contains("blake3::hash(vertex_bytes)"));
}

#[test]
fn sdf_prepare_report_summarizes_atlas_bake_and_vertices() {
    let plan = plan_sdf_atlas(&[text_batch("AB", UiFrame::new(8.0, 12.0, 64.0, 20.0))]);
    let bake_report = super::SdfAtlasBakeReport {
        slot_count: 2,
        visible_glyph_count: 2,
        empty_glyph_count: 0,
        atlas_byte_len: 512 * 512,
        nonzero_pixel_count: 64,
        resident_font_count: 1,
        loaded_font_count: 1,
        generation_failure_count: 0,
        r8_byte_len: 512 * 512,
        rgba_byte_len: 0,
        offline_glyph_count: 0,
        dynamic_glyph_count: 2,
        offline_resident_manifest_count: 0,
        offline_resident_artifact_identity_count: 0,
        offline_resident_artifact_byte_count: 0,
        offline_resident_glyph_bitmap_count: 0,
        offline_resident_glyph_bitmap_byte_count: 0,
        offline_manifest_parse_count: 0,
        offline_artifact_stat_count: 0,
        offline_artifact_read_count: 0,
        offline_artifact_read_byte_count: 0,
        offline_artifact_decode_count: 0,
        offline_pixel_copy_count: 0,
        offline_pixel_copy_byte_count: 0,
        offline_manifest_eviction_count: 0,
        offline_artifact_eviction_count: 0,
        offline_glyph_bitmap_eviction_count: 0,
        offline_oldest_artifact_idle_access_count: 0,
        offline_oldest_glyph_bitmap_idle_access_count: 0,
        resident_baked_glyph_count: 2,
        resident_baked_glyph_byte_count: 64,
        baked_glyph_eviction_count: 0,
        oldest_baked_glyph_idle_access_count: 1,
        resident_source_context_count: 1,
        resident_source_byte_count: 1_024,
        source_context_created_count: 1,
        source_context_eviction_count: 0,
        oldest_source_context_idle_access_count: 0,
        source_hash_count: 1,
        face_parse_count: 1,
        generation_batch_count: 1,
        generation_requested_glyph_count: 2,
        generation_unique_glyph_count: 2,
        generation_duplicate_glyph_count: 0,
        bitmap_clone_byte_count: 0,
        resident_atlas_page_count: 1,
        atlas_page_alloc_count: 1,
        atlas_page_zero_byte_count: 512 * 512,
        atlas_page_clear_count: 0,
        atlas_page_clear_byte_count: 0,
        atlas_page_write_count: 2,
        atlas_page_write_byte_count: 64,
        atlas_page_reused_slot_count: 0,
        atlas_full_page_scan_byte_count: 0,
        compiled_atlas_build_count: 1,
        compiled_atlas_reuse_count: 0,
        generation_scheduler: crate::text::sdf::SdfGenerationSchedulerDiagnostics::default(),
    };

    let cache_report = SdfAtlasCacheReport {
        previous_slot_count: 0,
        current_slot_count: 2,
        retained_slot_count: 0,
        stable_slot_count: 0,
        relocated_slot_count: 0,
        added_slot_count: 2,
        evicted_slot_count: 0,
        atlas_resized: true,
        dirty_rect: Some(crate::text::sdf::SdfAtlasRect {
            x: 0,
            y: 0,
            width: 128,
            height: 64,
        }),
        dirty_pages: vec![
            crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasDirtyPageReport {
                page_key: crate::text::atlas::GlyphAtlasPageKey::new(
                    crate::text::atlas::GlyphAtlasFormat::Sdf,
                    0,
                ),
                dirty_rect: crate::text::sdf::SdfAtlasRect {
                    x: 0,
                    y: 0,
                    width: 128,
                    height: 64,
                },
            },
        ],
    };

    let upload_report = sdf_atlas_upload_report(&plan, cache_report, true, 512 * 512, true);
    let draw_plan = SdfTextMaterialDrawPlan::default();
    let report = sdf_prepare_report(
        1,
        &plan,
        true,
        1,
        1,
        bake_report,
        upload_report,
        12,
        SdfVertexBufferWriteReport {
            capacity_byte_len: 4 * 1024,
            create_count: 1,
            write_byte_len: 720,
        },
        false,
        false,
        0,
        &draw_plan,
    );

    assert_eq!(
        report,
        ScreenSpaceUiSdfPrepareReport {
            text_batch_count: 1,
            atlas_slot_count: 2,
            atlas_size: plan.atlas_size,
            atlas_page_count: 1,
            msdf_atlas_page_count: 1,
            atlas_allocation_failure_count: 0,
            atlas_page_limit_failure_count: 0,
            atlas_oversized_failure_count: 0,
            atlas_resized: true,
            bake: bake_report,
            atlas_upload_byte_len: 512 * 512,
            atlas_upload_full_texture: true,
            atlas_upload: SdfAtlasUploadReport {
                mode: SdfAtlasUploadMode::FullTexture,
                byte_len: 512 * 512,
                full_texture: true,
                dirty_slot_count: 2,
                dirty_rect: Some(crate::text::sdf::SdfAtlasRect {
                    x: 0,
                    y: 0,
                    width: 512,
                    height: 512,
                },),
                dirty_byte_len: 512 * 512,
                dirty_pages: vec![SdfAtlasUploadPageReport {
                    page_key: crate::text::atlas::GlyphAtlasPageKey::new(
                        crate::text::atlas::GlyphAtlasFormat::Sdf,
                        0,
                    ),
                    dirty_rect: crate::text::sdf::SdfAtlasRect {
                        x: 0,
                        y: 0,
                        width: 512,
                        height: 512,
                    },
                    byte_len: 512 * 512,
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
            material_count: 0,
            draw_count: 0,
            outline_batch_count: 0,
            shadow_batch_count: 0,
            glow_batch_count: 0,
        }
    );
}

#[test]
fn sdf_prepare_report_summarizes_atlas_allocation_failures() {
    let mut plan = synthetic_layered_plan(0);
    plan.allocation_failures = vec![
        allocation_failure('B', SdfAtlasAllocationFailureReason::PageLimit),
        allocation_failure('C', SdfAtlasAllocationFailureReason::OversizedSlot),
    ];

    let report = sdf_prepare_report(
        1,
        &plan,
        false,
        1,
        1,
        SdfAtlasBakeReport::default(),
        SdfAtlasUploadReport::default(),
        0,
        SdfVertexBufferWriteReport::default(),
        false,
        false,
        0,
        &SdfTextMaterialDrawPlan::default(),
    );

    assert_eq!(report.atlas_allocation_failure_count, 2);
    assert_eq!(report.atlas_page_limit_failure_count, 1);
    assert_eq!(report.atlas_oversized_failure_count, 1);
}
