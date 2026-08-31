use super::*;

#[test]
fn sdf_atlas_quality_controls_slot_size_and_min_grid() {
    let plan = plan_sdf_atlas_with_quality(
        &[text_batch("AB", UiFrame::new(0.0, 0.0, 24.0, 12.0))],
        SdfAtlasQuality {
            slot_size_px: 32,
            min_grid_side: 2,
            max_cached_slot_count: 8,
        },
    );

    assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(64));
    assert_eq!(plan.slots.len(), 2);
    assert_eq!(plan.slots[0].rect.width, 32);
    assert_eq!(plan.slots[0].rect.height, 32);
    assert_eq!(plan.slots[1].rect.x, 32);
    assert_eq!(plan.slots[1].rect.y, 0);
}

#[test]
fn render_text_sdf_atlas_uses_unified_shelf_rects_on_sdf_page() {
    let plan = plan_sdf_atlas_with_quality(
        &[text_batch("ABCDEFG", UiFrame::new(0.0, 0.0, 84.0, 12.0))],
        SdfAtlasQuality {
            slot_size_px: 32,
            min_grid_side: 4,
            max_cached_slot_count: 16,
        },
    );
    let sdf_page = plan
        .atlas_set
        .page(GlyphAtlasFormat::Sdf, 0)
        .expect("SDF page should be registered before slot allocation is consumed");

    assert_eq!(
        sdf_page.key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(sdf_page.size, plan.atlas_size);
    assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(128));
    assert_eq!(plan.slots[0].rect, sdf_rect(0, 0, 32, 32));
    assert_eq!(plan.slots[3].rect, sdf_rect(96, 0, 32, 32));
    assert_eq!(plan.slots[4].rect, sdf_rect(0, 32, 32, 32));
}

#[test]
fn render_text_sdf_atlas_allocates_shelf_overflow_on_multiple_pages() {
    let plan = plan_sdf_atlas_with_quality(
        &[text_batch("ABCDEF", UiFrame::new(0.0, 0.0, 84.0, 12.0))],
        SdfAtlasQuality {
            slot_size_px: 32,
            min_grid_side: 2,
            max_cached_slot_count: 16,
        },
    );

    assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(64));
    assert_eq!(plan.atlas_set.page_count(), 2);
    assert_eq!(sdf_atlas_layer_count(&plan), 2);
    assert_eq!(
        plan.atlas_set
            .page(GlyphAtlasFormat::Sdf, 0)
            .map(|page| page.size),
        Some(crate::core::math::UVec2::splat(64))
    );
    assert_eq!(
        plan.atlas_set
            .page(GlyphAtlasFormat::Sdf, 1)
            .map(|page| page.storage_format),
        Some(GlyphAtlasStorageFormat::R8Unorm)
    );
    assert_eq!(
        plan.slots[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(plan.slots[0].rect, sdf_rect(0, 0, 32, 32));
    assert_eq!(
        plan.slots[3].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(plan.slots[3].rect, sdf_rect(32, 32, 32, 32));
    assert_eq!(
        plan.slots[4].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 1)
    );
    assert_eq!(plan.slots[4].rect, sdf_rect(0, 0, 32, 32));
    assert_eq!(
        plan.runs[0].glyph_slot_indices,
        glyph_slots(&[0, 1, 2, 3, 4, 5])
    );
}

#[test]
fn sdf_atlas_plan_uses_additional_pages_after_default_page_overflow() {
    let many_glyphs = (0..70)
        .map(|index| char::from_u32(0x1000 + index).unwrap())
        .collect::<String>();

    let plan = plan_sdf_atlas(&[text_batch(
        &many_glyphs,
        UiFrame::new(0.0, 0.0, 4096.0, 12.0),
    )]);

    assert_eq!(plan.slots.len(), 70);
    assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(512));
    assert_eq!(plan.atlas_set.page_count(), 2);
    assert_eq!(sdf_atlas_layer_count(&plan), 2);
    assert_eq!(
        plan.slots[63].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(plan.slots[63].rect, sdf_rect(448, 448, 64, 64));
    assert_eq!(
        plan.slots[64].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 1)
    );
    assert_eq!(plan.slots[64].rect, sdf_rect(0, 0, 64, 64));
}

#[test]
fn sdf_atlas_plan_reports_page_limit_allocation_failures() {
    let many_glyphs = glyph_range_string(0x1000, GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT * 4 + 2);

    let plan = plan_sdf_atlas_with_quality(
        &[text_batch(
            &many_glyphs,
            UiFrame::new(0.0, 0.0, 4096.0, 12.0),
        )],
        SdfAtlasQuality {
            slot_size_px: 32,
            min_grid_side: 2,
            max_cached_slot_count: 64,
        },
    );

    assert_eq!(plan.atlas_size, UVec2::splat(64));
    assert_eq!(
        plan.atlas_set.page_count(),
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT
    );
    assert_eq!(
        sdf_atlas_layer_count(&plan),
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT as u32
    );
    assert_eq!(
        plan.slots.len(),
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT * 4
    );
    assert_eq!(plan.allocation_failures.len(), 2);
    assert!(
        plan.allocation_failures
            .iter()
            .all(|failure| failure.reason == SdfAtlasAllocationFailureReason::PageLimit)
    );
    assert!(
        plan.allocation_failures
            .iter()
            .all(|failure| failure.requested_size == UVec2::splat(32))
    );
    assert!(
        plan.allocation_failures
            .iter()
            .all(|failure| failure.atlas_size == UVec2::splat(64))
    );
    assert_eq!(
        plan.slots.last().map(|slot| slot.page_key),
        Some(GlyphAtlasPageKey::new(
            GlyphAtlasFormat::Sdf,
            GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT as u32 - 1,
        ))
    );
    assert_eq!(
        plan.runs[0]
            .glyph_slot_indices
            .iter()
            .filter(|slot| slot.is_some())
            .count(),
        GLYPH_ATLAS_DEFAULT_MAX_PAGES_PER_FORMAT * 4
    );
    assert_eq!(&plan.runs[0].glyph_slot_indices[32..], &[None, None]);
    assert_eq!(
        &plan.runs[0].glyph_failure_reasons[32..],
        &[
            Some(SdfAtlasAllocationFailureReason::PageLimit),
            Some(SdfAtlasAllocationFailureReason::PageLimit),
        ]
    );
    assert_eq!(plan.runs[0].allocation_failure_count, 2);
    assert_eq!(plan.runs[0].page_limit_failure_count, 2);
    assert_eq!(plan.runs[0].oversized_failure_count, 0);
}

#[test]
fn sdf_atlas_plan_reports_oversized_slot_allocation_failure() {
    let key = glyph_key('A');

    let (atlas_set, slots, rebuilt_pages, allocation_failures) = allocate_sdf_atlas_slots(
        vec![key.clone()],
        UVec2::splat(32),
        SdfAtlasQuality {
            slot_size_px: 64,
            min_grid_side: 1,
            max_cached_slot_count: 1,
        },
    );

    assert_eq!(atlas_set.page_count(), 0);
    assert!(slots.is_empty());
    assert!(rebuilt_pages.is_empty());
    assert_eq!(
        allocation_failures,
        vec![SdfAtlasAllocationFailure {
            key,
            reason: SdfAtlasAllocationFailureReason::OversizedSlot,
            requested_size: UVec2::splat(64),
            atlas_size: UVec2::splat(32),
        }]
    );
}
