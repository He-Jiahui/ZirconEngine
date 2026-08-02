use super::*;

#[test]
fn sdf_atlas_plan_deduplicates_glyph_slots_across_batches() {
    let plan = plan_sdf_atlas(&[
        text_batch("ABBA", UiFrame::new(10.0, 20.0, 40.0, 12.0)),
        text_batch("CAB", UiFrame::new(10.0, 36.0, 30.0, 12.0)),
    ]);

    assert_eq!(plan.atlas_size, crate::core::math::UVec2::splat(512));
    assert_eq!(plan.slots.len(), 3);
    assert_eq!(plan.slots[0].key.glyph, 'A');
    assert_eq!(plan.slots[0].rect.x, 0);
    assert_eq!(plan.slots[0].rect.y, 0);
    assert_eq!(plan.slots[1].key.glyph, 'B');
    assert_eq!(plan.slots[1].rect.x, 64);
    assert_eq!(plan.slots[1].rect.y, 0);
    assert_eq!(plan.slots[2].key.glyph, 'C');
    assert_eq!(plan.slots[2].rect.x, 128);
    assert_eq!(plan.slots[2].rect.y, 0);
    assert_eq!(plan.runs.len(), 2);
    assert_eq!(plan.runs[0].glyph_slot_indices, glyph_slots(&[0, 1, 1, 0]));
    assert_eq!(plan.runs[1].glyph_slot_indices, glyph_slots(&[2, 0, 1]));
}

#[test]
fn sdf_atlas_plan_keys_glyph_slots_by_font_identity_and_fixed_bake_params() {
    let mut small_default = text_batch("A", UiFrame::new(0.0, 0.0, 12.0, 12.0));
    small_default.font_size = 12.0;
    let mut large_default = text_batch("A", UiFrame::new(0.0, 16.0, 24.0, 24.0));
    large_default.font_size = 24.0;
    let mut bold_default = text_batch("A", UiFrame::new(0.0, 32.0, 24.0, 24.0));
    bold_default.font_weight = 700;
    let mut small_icon = text_batch("A", UiFrame::new(0.0, 48.0, 12.0, 12.0));
    small_icon.font = Some("res://fonts/icons.font.toml".to_string());
    small_icon.font_family = Some("Zircon Icons".to_string());

    let plan = plan_sdf_atlas(&[small_default, large_default, bold_default, small_icon]);

    assert_eq!(plan.slots.len(), 3);
    assert_eq!(plan.slots[0].key.glyph, 'A');
    assert_eq!(
        plan.slots[0].key.font.as_deref(),
        Some("res://fonts/default.font.toml")
    );
    assert_eq!(
        plan.slots[0].key.font_family.as_deref(),
        Some("Zircon Sans")
    );
    assert_eq!(plan.slots[0].key.font_weight, 400);
    assert_eq!(plan.slots[0].key.bake_params, SdfBakeParams::default());
    assert_eq!(plan.slots[1].key.glyph, 'A');
    assert_eq!(
        plan.slots[1].key.font.as_deref(),
        Some("res://fonts/default.font.toml")
    );
    assert_eq!(
        plan.slots[1].key.font_family.as_deref(),
        Some("Zircon Sans")
    );
    assert_eq!(plan.slots[1].key.font_weight, 700);
    assert_eq!(plan.slots[1].key.bake_params, SdfBakeParams::default());
    assert_eq!(plan.slots[2].key.glyph, 'A');
    assert_eq!(
        plan.slots[2].key.font.as_deref(),
        Some("res://fonts/icons.font.toml")
    );
    assert_eq!(
        plan.slots[2].key.font_family.as_deref(),
        Some("Zircon Icons")
    );
    assert_eq!(plan.slots[2].key.font_weight, 400);
    assert_eq!(plan.runs[0].glyph_slot_indices, glyph_slots(&[0]));
    assert_eq!(plan.runs[1].glyph_slot_indices, glyph_slots(&[0]));
    assert_eq!(plan.runs[2].glyph_slot_indices, glyph_slots(&[1]));
    assert_eq!(plan.runs[3].glyph_slot_indices, glyph_slots(&[2]));
}

#[test]
fn sdf_atlas_plan_keeps_sdf_msdf_and_mtsdf_identity_and_storage_distinct() {
    let keys = vec![
        glyph_key_for_mode('A', SdfMode::Sdf),
        glyph_key_for_mode('A', SdfMode::Msdf),
        glyph_key_for_mode('A', SdfMode::Mtsdf),
    ];
    let plan = plan_sdf_atlas_from_slot_keys(
        keys.clone(),
        vec![keys.into_iter().map(Some).collect()],
        SdfAtlasQuality::default(),
    );

    assert_eq!(plan.slots.len(), 3);
    assert_eq!(plan.runs[0].glyph_slot_indices, glyph_slots(&[0, 1, 2]));
    assert_eq!(plan.slots[0].key.bake_params.mode, SdfMode::Sdf);
    assert_eq!(plan.slots[0].page_key.format, GlyphAtlasFormat::Sdf);
    assert_eq!(plan.slots[1].key.bake_params.mode, SdfMode::Msdf);
    assert_eq!(plan.slots[1].page_key.format, GlyphAtlasFormat::Msdf);
    assert_eq!(plan.slots[2].key.bake_params.mode, SdfMode::Mtsdf);
    assert_eq!(plan.slots[2].page_key.format, GlyphAtlasFormat::Msdf);
    assert_ne!(plan.slots[1].key, plan.slots[2].key);

    let sdf_page = plan.atlas_set.page(GlyphAtlasFormat::Sdf, 0).unwrap();
    let msdf_page = plan.atlas_set.page(GlyphAtlasFormat::Msdf, 0).unwrap();
    assert_eq!(sdf_page.storage_format, GlyphAtlasStorageFormat::R8Unorm);
    assert_eq!(
        msdf_page.storage_format,
        GlyphAtlasStorageFormat::Rgba8Unorm
    );
}

#[test]
fn sdf_atlas_plan_separates_locale_sensitive_glyph_slots() {
    let mut simplified = text_batch("界", UiFrame::new(0.0, 0.0, 16.0, 16.0));
    simplified.language = Some("zh-Hans".to_string());
    let mut japanese = text_batch("界", UiFrame::new(0.0, 16.0, 16.0, 16.0));
    japanese.language = Some("ja".to_string());

    let plan = plan_sdf_atlas(&[simplified, japanese]);

    assert_eq!(plan.slots.len(), 2);
    assert_eq!(plan.slots[0].key.language.as_deref(), Some("ja"));
    assert_eq!(plan.slots[1].key.language.as_deref(), Some("zh-hans"));
}

#[test]
fn sdf_atlas_plan_preserves_shaped_glyph_and_face_identity() {
    let mut vertical = text_batch("。", UiFrame::new(0.0, 0.0, 32.0, 48.0));
    vertical.writing_mode = UiTextWritingMode::VerticalRl;
    vertical.shaped_glyphs = vec![ScreenSpaceUiShapedGlyph {
        glyph_id: 321,
        font_id: Some(TextFontFaceHandle::new(17, 5)),
        font_instance_id: Some(TextFontFaceHandle::new(29, 5)),
        source_scalar: '。',
        source_range: UiTextRange {
            start: 0,
            end: "。".len(),
        },
        advance: 30.0,
        offset_x: -15.0,
        offset_y: 27.0,
        rotation: ShapedGlyphRotation::None,
        requires_atlas_slot: true,
    }];

    let plan = plan_sdf_atlas(&[vertical]);

    assert_eq!(plan.slots.len(), 1);
    assert_eq!(plan.slots[0].key.glyph, '。');
    assert_eq!(plan.slots[0].key.glyph_id, Some(321));
    assert_eq!(
        plan.slots[0].key.font_id,
        Some(TextFontFaceHandle::new(17, 5))
    );
    assert_eq!(
        plan.slots[0].key.font_instance_id,
        Some(TextFontFaceHandle::new(29, 5))
    );
    assert_eq!(plan.runs[0].glyph_slot_indices, vec![Some(0)]);
}

#[test]
fn sdf_atlas_plan_separates_variable_font_instances_on_same_face() {
    let mut regular = text_batch("A", UiFrame::new(0.0, 0.0, 32.0, 48.0));
    regular.shaped_glyphs = vec![ScreenSpaceUiShapedGlyph {
        glyph_id: 41,
        font_id: Some(TextFontFaceHandle::new(17, 5)),
        font_instance_id: Some(TextFontFaceHandle::new(29, 5)),
        source_scalar: 'A',
        source_range: UiTextRange { start: 0, end: 1 },
        advance: 20.0,
        offset_x: 0.0,
        offset_y: 0.0,
        rotation: ShapedGlyphRotation::None,
        requires_atlas_slot: true,
    }];
    let mut expanded = regular.clone();
    expanded.shaped_glyphs[0].font_instance_id = Some(TextFontFaceHandle::new(30, 5));

    let plan = plan_sdf_atlas(&[regular, expanded]);

    assert_eq!(plan.slots.len(), 2);
    assert_ne!(
        plan.slots[0].key.font_instance_id,
        plan.slots[1].key.font_instance_id
    );
}

#[test]
fn render_text_sdf_atlas_unified_with_alpha() {
    let plan = plan_sdf_atlas(&[text_batch("SDF", UiFrame::new(0.0, 0.0, 48.0, 16.0))]);
    let alpha_page = GlyphAtlasPageSpec::new(
        GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
        plan.atlas_size,
    );
    let unified = plan.atlas_set.clone().with_page(alpha_page);

    let sdf_page = unified
        .page(GlyphAtlasFormat::Sdf, 0)
        .expect("SDF atlas page should be registered in the unified atlas set");
    let alpha_page = unified
        .page(GlyphAtlasFormat::AlphaMask, 0)
        .expect("alpha atlas page should share the unified atlas set");

    assert_eq!(unified.page_count(), 2);
    assert_eq!(sdf_page.key.format, GlyphAtlasFormat::Sdf);
    assert_eq!(alpha_page.key.format, GlyphAtlasFormat::AlphaMask);
    assert_ne!(sdf_page.key, alpha_page.key);
    assert_eq!(sdf_page.size, plan.atlas_size);
    assert_eq!(alpha_page.size, plan.atlas_size);
    assert_eq!(sdf_page.storage_format, GlyphAtlasStorageFormat::R8Unorm);
    assert_eq!(alpha_page.storage_format, GlyphAtlasStorageFormat::R8Unorm);
}

#[test]
fn sdf_atlas_plan_preserves_whitespace_advances_without_slots() {
    let plan = plan_sdf_atlas(&[text_batch("A B", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);

    assert_eq!(plan.slots.len(), 2);
    assert_eq!(plan.slots[0].key.glyph, 'A');
    assert_eq!(plan.slots[1].key.glyph, 'B');
    assert_eq!(
        plan.runs[0].glyph_slot_indices,
        vec![Some(0), None, Some(1)]
    );
}

#[test]
fn sdf_atlas_records_generation_failures_by_verified_bake_slot_index() {
    let mut atlas = ScreenSpaceUiSdfAtlas::new();
    atlas.prepare(&[text_batch("AB", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);
    let first_key = atlas.plan().slots[0].key.clone();
    let second_key = atlas.plan().slots[1].key.clone();
    let expected = SdfGlyphGenerationError::MissingGlyphOutline(2);

    let failures: Arc<[SdfAtlasGlyphGenerationFailure]> = vec![
        SdfAtlasGlyphGenerationFailure {
            slot_index: 0,
            key: second_key.clone(),
            error: SdfGlyphGenerationError::MissingGlyphOutline(99),
        },
        SdfAtlasGlyphGenerationFailure {
            slot_index: 1,
            key: second_key,
            error: expected,
        },
        SdfAtlasGlyphGenerationFailure {
            slot_index: usize::MAX,
            key: first_key,
            error: SdfGlyphGenerationError::GenerationBudgetDeferred,
        },
    ]
    .into();
    atlas.record_generation_failures(&failures);

    assert_eq!(
        atlas.plan().runs[0].glyph_generation_failures,
        vec![None, Some(expected)]
    );
    assert_eq!(atlas.plan().runs[0].generation_failure_count, 1);
    let first_failure_slots = atlas.plan().runs[0].glyph_generation_failures.as_ptr();

    atlas.record_generation_failures(&failures);

    assert_eq!(
        atlas.plan().runs[0].glyph_generation_failures.as_ptr(),
        first_failure_slots
    );
}

#[test]
fn sdf_atlas_plan_keeps_format_controls_out_of_slots() {
    let plan = plan_sdf_atlas(&[text_batch(
        "A\u{200D}\u{FE0F}B",
        UiFrame::new(10.0, 20.0, 40.0, 12.0),
    )]);

    assert_eq!(plan.slots.len(), 2);
    assert_eq!(plan.slots[0].key.glyph, 'A');
    assert_eq!(plan.slots[1].key.glyph, 'B');
    assert_eq!(
        plan.runs[0].glyph_slot_indices,
        vec![Some(0), None, None, Some(1)]
    );
    assert_eq!(plan.runs[0].glyph_failure_reasons, vec![None; 4]);
    assert_eq!(plan.runs[0].allocation_failure_count, 0);
}

#[test]
fn sdf_atlas_run_records_failure_reasons_per_glyph() {
    let allocated = glyph_key('A');
    let page_limited = glyph_key('B');
    let oversized = glyph_key('C');

    let run = sdf_atlas_run_for_glyph_keys(
        vec![
            Some(allocated.clone()),
            None,
            Some(page_limited.clone()),
            Some(oversized.clone()),
        ],
        &HashMap::from([(allocated, 7)]),
        &HashMap::from([
            (page_limited, SdfAtlasAllocationFailureReason::PageLimit),
            (oversized, SdfAtlasAllocationFailureReason::OversizedSlot),
        ]),
    );

    assert_eq!(run.glyph_slot_indices, vec![Some(7), None, None, None]);
    assert_eq!(
        run.glyph_failure_reasons,
        vec![
            None,
            None,
            Some(SdfAtlasAllocationFailureReason::PageLimit),
            Some(SdfAtlasAllocationFailureReason::OversizedSlot),
        ]
    );
    assert_eq!(run.allocation_failure_count, 2);
    assert_eq!(run.page_limit_failure_count, 1);
    assert_eq!(run.oversized_failure_count, 1);
}

#[test]
fn sdf_atlas_plan_assigns_slot_rects_by_key_not_batch_order() {
    let first = plan_sdf_atlas(&[text_batch("AB", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);
    let second = plan_sdf_atlas(&[text_batch("BA", UiFrame::new(10.0, 20.0, 40.0, 12.0))]);

    assert_eq!(first.slots, second.slots);
}
