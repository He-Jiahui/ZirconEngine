use super::*;
use crate::text::font::shared_font_database_test_read_guard;

#[test]
fn sdf_font_bake_report_distinguishes_newly_loaded_and_resident_fonts() {
    let _shared_font_database = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['A']);

    let first = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );
    let second = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    assert!(first.report.resident_font_count >= 1);
    assert_eq!(
        first.report.loaded_font_count,
        first.report.resident_font_count
    );
    assert_eq!(
        second.report.resident_font_count,
        first.report.resident_font_count
    );
    assert_eq!(second.report.loaded_font_count, 0);
}

#[test]
fn sdf_font_bake_packs_mixed_formats_and_reuses_mode_keyed_cache() {
    let _shared_font_database = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_mixed_distance_fields();

    let first = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );
    let cached_glyph_count = bake.glyphs.len();
    let second = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    let page_area = (plan.atlas_size.x * plan.atlas_size.y) as usize;
    assert_eq!(first.pixels.len(), page_area + page_area * 4);
    assert_eq!(first.report.r8_byte_len, page_area);
    assert_eq!(first.report.rgba_byte_len, page_area * 4);
    assert_eq!(first.report.atlas_byte_len, first.pixels.len());
    assert_eq!(first.report.visible_glyph_count, 3);
    assert_eq!(first.report.generation_failure_count, 1);
    assert_eq!(first.generation_failures.len(), 1);
    assert_eq!(first.generation_failures[0].slot_index, 3);
    assert_eq!(
        first.generation_failures[0].key.bake_params.mode,
        SdfMode::Msdf
    );
    assert!(matches!(
        first.generation_failures[0].error,
        SdfGlyphGenerationError::MissingGlyphOutline(_)
    ));
    assert_eq!(
        first.pages,
        vec![
            SdfAtlasBakePage {
                page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0),
                source_offset: 0,
                byte_len: page_area,
            },
            SdfAtlasBakePage {
                page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Msdf, 0),
                source_offset: page_area,
                byte_len: page_area * 4,
            },
        ]
    );

    let sdf = slot_pixels_for_bake_page(&first, plan.atlas_size.x, 0, plan.slots[0].rect);
    let msdf = slot_pixels_for_bake_page(
        &first,
        plan.atlas_size.x,
        1,
        baked_glyph_rect(plan.slots[1].rect, &first.glyphs[1]),
    );
    let mtsdf = slot_pixels_for_bake_page(
        &first,
        plan.atlas_size.x,
        1,
        baked_glyph_rect(plan.slots[2].rect, &first.glyphs[2]),
    );
    assert!(sdf.iter().any(|sample| *sample != 0));
    assert!(msdf
        .chunks_exact(4)
        .filter(|sample| sample[0] != 0 || sample[1] != 0 || sample[2] != 0)
        .all(|sample| sample[3] == u8::MAX));
    assert!(msdf
        .chunks_exact(4)
        .any(|sample| sample[0] != sample[1] || sample[1] != sample[2]));
    assert!(mtsdf.chunks_exact(4).any(|sample| sample[3] != u8::MAX));
    assert!(mtsdf.chunks_exact(4).any(|sample| {
        let mut rgb = [sample[0], sample[1], sample[2]];
        rgb.sort_unstable();
        sample[3] != rgb[1]
    }));

    assert_eq!(bake.glyphs.len(), cached_glyph_count);
    assert_eq!(second.pixels, first.pixels);
    assert_eq!(second.pages, first.pages);
    assert_eq!(second.generation_failures, first.generation_failures);
    let mut expected_second_report = first.report;
    expected_second_report.loaded_font_count = 0;
    assert_eq!(second.report, expected_second_report);
}

#[cfg(target_os = "windows")]
#[test]
fn sdf_font_bake_rasterizes_materialized_system_cjk_face() {
    let (generation, shared_font_database) = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = shared_font_database.clone();
    let asset_manager = ProjectAssetManager::default();
    let face = font_database
        .match_face(&FontQuery::single_family("Microsoft YaHei UI"))
        .expect("Windows CJK system font")
        .face;
    assert!(bake.ensure_sdf_font(face, &font_database));

    let mut plan = atlas_plan_for_glyphs(&['本']);
    plan.slots[0].key.font_id = Some(
        crate::text::font::register_font_face_handle(face, generation)
            .expect("system face Text handle"),
    );
    plan.slots[0].key.font_family = Some("Microsoft YaHei UI".to_string());
    plan.slots[0].key.language = Some("zh-Hans".to_string());
    let atlas = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    assert_eq!(atlas.report.slot_count, 1);
    assert_eq!(atlas.report.visible_glyph_count, 1);
    assert_eq!(atlas.report.empty_glyph_count, 0);
    assert!(atlas.report.nonzero_pixel_count > 0);
    assert_eq!(atlas.report.resident_font_count, 1);
    assert_eq!(atlas.report.loaded_font_count, 0);
}

#[cfg(target_os = "windows")]
#[test]
fn sdf_font_bake_prefers_shaped_glyph_id_on_authoritative_face() {
    let (generation, shared_font_database) = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let font_database = shared_font_database.clone();
    let face = font_database
        .match_face(&FontQuery::single_family("Microsoft YaHei UI"))
        .expect("Windows CJK system font")
        .face;
    assert!(bake.ensure_sdf_font(face, &font_database));
    let font = bake.fonts.get(&face).expect("materialized SDF font");
    let shaped_id = font.lookup_glyph_index('布');
    let scalar_id = font.lookup_glyph_index('。');
    assert_ne!(shaped_id, 0);
    assert_ne!(shaped_id, scalar_id);
    let key = SdfAtlasGlyphKey {
        glyph: '。',
        glyph_id: Some(shaped_id as u32),
        font_id: Some(
            crate::text::font::register_font_face_handle(face, generation)
                .expect("system face Text handle"),
        ),
        font_instance_id: None,
        font: Some(DEFAULT_FONT_ASSET.to_string()),
        font_family: Some("Microsoft YaHei UI".to_string()),
        language: Some("zh-hans".to_string()),
        font_weight: FontWeight::NORMAL.0,
        bake_params: SdfBakeParams::default(),
    };

    assert_eq!(glyph_index(font, &key, face, &font_database), shaped_id);
}
