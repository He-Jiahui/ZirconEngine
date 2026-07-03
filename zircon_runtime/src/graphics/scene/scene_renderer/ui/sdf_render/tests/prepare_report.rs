use super::*;

#[test]
fn sdf_prepare_report_summarizes_atlas_bake_and_vertices() {
    let plan = plan_sdf_atlas(&[text_batch("AB", UiFrame::new(8.0, 12.0, 64.0, 20.0))]);
    let bake_report = super::SdfAtlasBakeReport {
        slot_count: 2,
        visible_glyph_count: 2,
        empty_glyph_count: 0,
        atlas_byte_len: 512 * 512,
        nonzero_pixel_count: 64,
        loaded_font_count: 1,
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
        dirty_rect: Some(
            crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasRect {
                x: 0,
                y: 0,
                width: 128,
                height: 64,
            },
        ),
        dirty_pages: vec![
            crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasDirtyPageReport {
                page_key: crate::graphics::text::atlas::GlyphAtlasPageKey::new(
                    crate::graphics::text::atlas::GlyphAtlasFormat::Sdf,
                    0,
                ),
                dirty_rect: crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasRect {
                    x: 0,
                    y: 0,
                    width: 128,
                    height: 64,
                },
            },
        ],
    };

    let upload_report = sdf_atlas_upload_report(&plan, cache_report, true, 512 * 512, true);
    let report = sdf_prepare_report(1, &plan, true, 1, bake_report, upload_report, 12);

    assert_eq!(
        report,
        ScreenSpaceUiSdfPrepareReport {
            text_batch_count: 1,
            atlas_slot_count: 2,
            atlas_size: plan.atlas_size,
            atlas_page_count: 1,
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
                dirty_rect: Some(
                    crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasRect {
                        x: 0,
                        y: 0,
                        width: 512,
                        height: 512,
                    },
                ),
                dirty_byte_len: 512 * 512,
                dirty_pages: vec![SdfAtlasUploadPageReport {
                    page_key: crate::graphics::text::atlas::GlyphAtlasPageKey::new(
                        crate::graphics::text::atlas::GlyphAtlasFormat::Sdf,
                        0,
                    ),
                    dirty_rect:
                        crate::graphics::scene::scene_renderer::ui::sdf_atlas::SdfAtlasRect {
                            x: 0,
                            y: 0,
                            width: 512,
                            height: 512,
                        },
                    byte_len: 512 * 512,
                }],
            },
            vertex_count: 12,
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
        SdfAtlasBakeReport::default(),
        SdfAtlasUploadReport::default(),
        0,
    );

    assert_eq!(report.atlas_allocation_failure_count, 2);
    assert_eq!(report.atlas_page_limit_failure_count, 1);
    assert_eq!(report.atlas_oversized_failure_count, 1);
}
