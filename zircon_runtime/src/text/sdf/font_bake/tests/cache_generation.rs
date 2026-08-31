use super::*;
use crate::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};
use crate::text::font::shared_font_database_test_read_guard;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct MissingFontAssetRun;

impl SdfTextRun for MissingFontAssetRun {
    fn font(&self) -> Option<&str> {
        Some("res://fonts/missing-prepared-atlas-report.font.toml")
    }

    fn font_family(&self) -> Option<&str> {
        None
    }

    fn language(&self) -> Option<&str> {
        None
    }

    fn font_weight(&self) -> u16 {
        FontWeight::NORMAL.0
    }

    fn font_size(&self) -> f32 {
        16.0
    }

    fn render_scalars(&self) -> Vec<char> {
        vec!['A']
    }

    fn resolved_glyph_advances(&self) -> Option<Vec<f32>> {
        None
    }

    fn shaped_glyph(&self, _glyph_index: usize) -> Option<SdfShapedGlyphIdentity> {
        None
    }
}

#[test]
fn sdf_generation_source_cache_has_explicit_context_and_byte_budgets() {
    let source = include_str!("../source_context.rs");

    assert!(source.contains("MAX_RESIDENT_SOURCE_CONTEXT_COUNT"));
    assert!(source.contains("MAX_RESIDENT_SOURCE_BYTE_COUNT"));
    assert!(source.contains("context_recency"));
    assert!(source.contains("context_eviction_count"));
    assert!(source.contains("resident_source_byte_count"));
}

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
fn sdf_font_asset_failure_report_is_resident_and_generation_scoped() {
    let _shared_font_database = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let missing = atlas_plan_for_asset('A', "res://fonts/missing-resident-report.font.toml");

    let first = bake.build_atlas_from_slots(
        missing.atlas_size,
        &missing.slots,
        &mut font_database,
        &asset_manager,
    );
    let second = bake.build_atlas_from_slots(
        missing.atlas_size,
        &missing.slots,
        &mut font_database,
        &asset_manager,
    );

    assert_eq!(first.report.resident_font_asset_error_count, 1);
    assert_eq!(
        first.report.resident_font_asset_no_registered_faces_count,
        1
    );
    assert_eq!(
        second.report.resident_font_asset_error_count,
        first.report.resident_font_asset_error_count
    );
    assert_eq!(
        second.report.resident_font_asset_no_registered_faces_count,
        first.report.resident_font_asset_no_registered_faces_count
    );

    bake.invalidate_faces();
    let cleared = SdfAtlasPlan::default();
    let after_invalidation = bake.build_atlas_from_slots(
        cleared.atlas_size,
        &cleared.slots,
        &mut font_database,
        &asset_manager,
    );

    assert_eq!(after_invalidation.report.resident_font_asset_error_count, 0);
    assert_eq!(
        after_invalidation
            .report
            .resident_font_asset_no_registered_faces_count,
        0
    );
}

#[test]
fn sdf_prepared_atlas_reuse_refreshes_resident_font_asset_failure_report() {
    let _shared_font_database = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let empty = SdfAtlasPlan::default();

    let initial = bake.build_atlas_from_slots(
        empty.atlas_size,
        &empty.slots,
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(initial.report.resident_font_asset_error_count, 0);

    let _ = bake.prepare_run_cpu(&MissingFontAssetRun, &mut font_database, &asset_manager);
    let reused = bake.build_atlas_from_slots(
        empty.atlas_size,
        &empty.slots,
        &mut font_database,
        &asset_manager,
    );

    assert_eq!(reused.report.resident_font_asset_error_count, 1);
    assert_eq!(
        reused.report.resident_font_asset_no_registered_faces_count,
        1
    );
    assert_eq!(reused.report.compiled_atlas_reuse_count, 1);
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
    assert_eq!(first.report.r8_byte_len, page_area);
    assert_eq!(first.report.rgba_byte_len, page_area * 4);
    assert_eq!(first.report.atlas_byte_len, page_area + page_area * 4);
    assert_eq!(first.report.visible_glyph_count, 3);
    assert_eq!(first.report.generation_failure_count, 1);
    assert_eq!(first.report.source_hash_count, 1);
    assert_eq!(first.report.face_parse_count, 1);
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
    assert_eq!(first.pages.len(), 2);
    assert_eq!(
        first.pages[0].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0)
    );
    assert_eq!(first.pages[0].source_offset, 0);
    assert_eq!(first.pages[0].byte_len, page_area);
    assert_eq!(
        first.pages[1].page_key,
        GlyphAtlasPageKey::new(GlyphAtlasFormat::Msdf, 0)
    );
    assert_eq!(first.pages[1].source_offset, page_area);
    assert_eq!(first.pages[1].byte_len, page_area * 4);
    assert_eq!(first.report.atlas_page_alloc_count, 2);
    assert_eq!(first.report.atlas_page_zero_byte_count, page_area * 5);
    assert_eq!(first.report.atlas_full_page_scan_byte_count, 0);

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
    assert!(
        msdf.chunks_exact(4)
            .filter(|sample| sample[0] != 0 || sample[1] != 0 || sample[2] != 0)
            .all(|sample| sample[3] == u8::MAX)
    );
    assert!(
        msdf.chunks_exact(4)
            .any(|sample| sample[0] != sample[1] || sample[1] != sample[2])
    );
    assert!(mtsdf.chunks_exact(4).any(|sample| sample[3] != u8::MAX));
    assert!(mtsdf.chunks_exact(4).any(|sample| {
        let mut rgb = [sample[0], sample[1], sample[2]];
        rgb.sort_unstable();
        sample[3] != rgb[1]
    }));

    assert_eq!(bake.glyphs.len(), cached_glyph_count);
    assert_eq!(second.pages, first.pages);
    assert!(Arc::ptr_eq(&first.pages, &second.pages));
    assert!(Arc::ptr_eq(&first.glyphs, &second.glyphs));
    assert!(Arc::ptr_eq(
        &first.generation_failures,
        &second.generation_failures
    ));
    assert!(second.dirty_pages.is_empty());
    assert!(
        first
            .pages
            .iter()
            .zip(second.pages.iter())
            .all(|(first, second)| Arc::ptr_eq(&first.pixels, &second.pixels))
    );
    assert_eq!(second.generation_failures, first.generation_failures);
    let mut expected_second_report = first.report;
    expected_second_report.loaded_font_count = 0;
    expected_second_report.source_context_created_count = 0;
    expected_second_report.source_hash_count = 0;
    expected_second_report.face_parse_count = 0;
    expected_second_report.generation_batch_count = 0;
    expected_second_report.generation_requested_glyph_count = 0;
    expected_second_report.generation_unique_glyph_count = 0;
    expected_second_report.generation_duplicate_glyph_count = 0;
    expected_second_report.offline_manifest_parse_count = 0;
    expected_second_report.offline_artifact_stat_count = 0;
    expected_second_report.offline_artifact_read_count = 0;
    expected_second_report.offline_artifact_read_byte_count = 0;
    expected_second_report.offline_artifact_decode_count = 0;
    expected_second_report.offline_pixel_copy_count = 0;
    expected_second_report.offline_pixel_copy_byte_count = 0;
    expected_second_report.atlas_page_alloc_count = 0;
    expected_second_report.atlas_page_zero_byte_count = 0;
    expected_second_report.atlas_page_clear_count = 0;
    expected_second_report.atlas_page_clear_byte_count = 0;
    expected_second_report.atlas_page_write_count = 0;
    expected_second_report.atlas_page_write_byte_count = 0;
    expected_second_report.atlas_page_reused_slot_count = plan.slots.len();
    expected_second_report.atlas_full_page_scan_byte_count = 0;
    expected_second_report.compiled_atlas_build_count = 0;
    expected_second_report.compiled_atlas_reuse_count = 1;
    assert_eq!(second.report, expected_second_report);
}

#[test]
fn sdf_font_bake_batches_unique_dynamic_glyphs_and_reuses_arc_bitmaps() {
    let _shared_font_database = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['A', 'M', 'A']);

    let first = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );
    let first_bitmap = Arc::clone(
        &bake
            .glyphs
            .get(&plan.slots[0].key)
            .expect("cached dynamic glyph")
            .bitmap,
    );
    let second = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    assert_eq!(first.report.source_context_created_count, 1);
    assert_eq!(first.report.source_hash_count, 1);
    assert_eq!(first.report.face_parse_count, 1);
    assert_eq!(first.report.generation_batch_count, 1);
    assert_eq!(first.report.generation_requested_glyph_count, 2);
    assert_eq!(first.report.generation_unique_glyph_count, 2);
    assert_eq!(first.report.bitmap_clone_byte_count, 0);
    assert_eq!(second.report.source_context_created_count, 0);
    assert_eq!(second.report.source_hash_count, 0);
    assert_eq!(second.report.face_parse_count, 0);
    assert_eq!(second.report.generation_batch_count, 0);
    assert_eq!(second.report.generation_requested_glyph_count, 0);
    assert_eq!(second.report.bitmap_clone_byte_count, 0);
    assert_eq!(second.pages, first.pages);
    assert!(Arc::ptr_eq(&first.pages[0].pixels, &second.pages[0].pixels));
    assert_eq!(second.report.atlas_page_alloc_count, 0);
    assert_eq!(second.report.atlas_page_zero_byte_count, 0);
    assert_eq!(second.report.atlas_page_clear_count, 0);
    assert_eq!(second.report.atlas_page_write_count, 0);
    assert_eq!(second.report.atlas_full_page_scan_byte_count, 0);
    assert!(Arc::ptr_eq(
        &first_bitmap,
        &bake
            .glyphs
            .get(&plan.slots[0].key)
            .expect("stable cached dynamic glyph")
            .bitmap
    ));
}

#[test]
fn sdf_baked_glyph_cache_evicts_oldest_entries_at_the_hard_count_limit() {
    let mut bake = SdfFontBakeCache::new();
    let base_key = atlas_plan_for_glyphs(&['A']).slots.remove(0).key;
    let mut first_key = None;
    let mut last_key = None;

    for index in 0..=4_096_u32 {
        let mut key = base_key.clone();
        key.glyph = char::from_u32(0x1_000 + index).expect("test Unicode scalar");
        let glyph = RawBakedGlyph {
            metrics: SdfGlyphMetrics::default(),
            bitmap: Arc::from([1_u8]),
            visible: true,
            generation_error: None,
            source: RawBakedGlyphSource::Dynamic,
        };
        if index == 0 {
            first_key = Some(key.clone());
        }
        last_key = Some(key.clone());
        bake.shaped_face_resolutions.insert(key.clone(), None);
        bake.insert_baked_glyph(key, glyph);
    }

    bake.enforce_baked_glyph_budget(&[]);
    let report = bake.report_baked_glyph_cache();

    assert_eq!(report.resident_count, 4_096);
    assert_eq!(report.resident_byte_count, 4_096);
    assert_eq!(report.eviction_count, 1);
    let first_key = first_key.expect("first key");
    let last_key = last_key.expect("last key");
    assert!(!bake.glyphs.contains_key(&first_key));
    assert!(!bake.shaped_face_resolutions.contains_key(&first_key));
    assert!(bake.glyphs.contains_key(&last_key));
    assert!(bake.shaped_face_resolutions.contains_key(&last_key));
}

#[test]
fn sdf_cpu_only_glyph_sidecars_evict_at_the_shared_hard_count_limit() {
    let mut bake = SdfFontBakeCache::new();
    let base_key = atlas_plan_for_glyphs(&['A']).slots.remove(0).key;
    let mut first_key = None;
    let mut last_key = None;

    for index in 0..=4_096_u32 {
        let mut key = base_key.clone();
        key.glyph = char::from_u32(0x2_000 + index).expect("test Unicode scalar");
        if index == 0 {
            first_key = Some(key.clone());
        }
        last_key = Some(key.clone());
        bake.measured_glyphs
            .insert(key.clone(), SdfGlyphMetrics::default());
        bake.face_resolutions.insert(key.clone(), Vec::new());
        bake.shaped_face_resolutions.insert(key.clone(), None);
        bake.touch_cached_glyph_key(key);
    }

    bake.enforce_baked_glyph_budget(&[]);
    let report = bake.report_baked_glyph_cache();

    let first_key = first_key.expect("first key");
    let last_key = last_key.expect("last key");
    assert!(bake.glyphs.is_empty());
    assert_eq!(report.resident_count, 4_096);
    assert_eq!(report.resident_byte_count, 0);
    assert_eq!(report.eviction_count, 1);
    assert_eq!(bake.measured_glyphs.len(), 4_096);
    assert_eq!(bake.face_resolutions.len(), 4_096);
    assert_eq!(bake.shaped_face_resolutions.len(), 4_096);
    assert!(!bake.measured_glyphs.contains_key(&first_key));
    assert!(!bake.face_resolutions.contains_key(&first_key));
    assert!(!bake.shaped_face_resolutions.contains_key(&first_key));
    assert!(bake.measured_glyphs.contains_key(&last_key));
    assert!(bake.face_resolutions.contains_key(&last_key));
    assert!(bake.shaped_face_resolutions.contains_key(&last_key));
}

#[test]
fn sdf_clear_cached_glyph_entries_removes_cpu_only_glyph_sidecars() {
    let mut bake = SdfFontBakeCache::new();
    let key = atlas_plan_for_glyphs(&['A']).slots.remove(0).key;
    bake.measured_glyphs
        .insert(key.clone(), SdfGlyphMetrics::default());
    bake.face_resolutions.insert(key.clone(), Vec::new());
    bake.shaped_face_resolutions.insert(key.clone(), None);
    bake.touch_cached_glyph_key(key);

    bake.clear_cached_glyph_entries();

    assert!(bake.glyphs.is_empty());
    assert!(bake.measured_glyphs.is_empty());
    assert!(bake.face_resolutions.is_empty());
    assert!(bake.shaped_face_resolutions.is_empty());
    assert!(bake.baked_glyph_recency.is_empty());
    assert!(bake.baked_glyph_recency_order.is_empty());
}

#[test]
fn sdf_prepared_atlas_refreshes_visible_glyph_recency_before_a_layout_change() {
    let _shared_font_database = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let hot_plan = atlas_plan_for_glyphs(&['A']);
    let next_plan = atlas_plan_for_glyphs(&['M']);
    let hot_key = hot_plan.slots[0].key.clone();
    let mut oldest_filler = None;

    bake.build_atlas_from_slots(
        hot_plan.atlas_size,
        &hot_plan.slots,
        &mut font_database,
        &asset_manager,
    );
    for index in 0..4_095_u32 {
        let mut key = hot_key.clone();
        key.glyph = char::from_u32(0x3_000 + index).expect("test Unicode scalar");
        if index == 0 {
            oldest_filler = Some(key.clone());
        }
        bake.insert_baked_glyph(
            key,
            RawBakedGlyph {
                metrics: SdfGlyphMetrics::default(),
                bitmap: Arc::from([1_u8]),
                visible: true,
                generation_error: None,
                source: RawBakedGlyphSource::Dynamic,
            },
        );
    }

    bake.build_atlas_from_slots(
        next_plan.atlas_size,
        &next_plan.slots,
        &mut font_database,
        &asset_manager,
    );

    assert!(bake.glyphs.contains_key(&hot_key));
    assert!(
        !bake
            .glyphs
            .contains_key(&oldest_filler.expect("oldest filler key"))
    );
}

#[test]
fn sdf_font_bake_scheduled_generation_falls_back_then_commits_next_frame() {
    let _shared_font_database = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['A', 'M']);
    let scheduler = SdfGenerationScheduler::new(
        TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(2)),
        SdfGenerationSchedulerOptions::new(4),
    );

    let first = bake.build_atlas_from_slots_scheduled(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
        &scheduler,
        1,
    );

    assert_eq!(first.generation_failures.len(), 2);
    assert!(
        first
            .generation_failures
            .iter()
            .all(|failure| failure.error == SdfGlyphGenerationError::GenerationPending)
    );
    assert_eq!(first.report.visible_glyph_count, 0);
    assert_eq!(first.report.generation_failure_count, 2);
    assert_eq!(
        first.report.generation_scheduler.in_flight_batch_count
            + first.report.generation_scheduler.completion_backlog_count,
        1
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut frame_index = 2_u64;
    let committed = loop {
        let frame = bake.build_atlas_from_slots_scheduled(
            plan.atlas_size,
            &plan.slots,
            &mut font_database,
            &asset_manager,
            &scheduler,
            frame_index,
        );
        if frame.generation_failures.is_empty() {
            break frame;
        }
        assert!(
            Instant::now() < deadline,
            "scheduled SDF generation deadline"
        );
        frame_index = frame_index.saturating_add(1);
        std::thread::yield_now();
    };

    assert_eq!(committed.report.visible_glyph_count, 2);
    assert_eq!(committed.report.generation_failure_count, 0);
    assert_eq!(committed.report.generation_batch_count, 1);
    assert_eq!(committed.report.generation_requested_glyph_count, 2);
    assert_eq!(committed.report.atlas_page_write_count, 2);
    assert!(!committed.dirty_pages.is_empty());
}

#[test]
fn sdf_font_bake_scheduler_budget_defer_does_not_poison_glyph_cache() {
    let _shared_font_database = shared_font_database_test_read_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['A']);
    let scheduler = SdfGenerationScheduler::new(
        TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1)),
        SdfGenerationSchedulerOptions::new(1).with_source_byte_budget(1),
    );

    let atlas = bake.build_atlas_from_slots_scheduled(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
        &scheduler,
        1,
    );

    assert_eq!(
        atlas.generation_failures[0].error,
        SdfGlyphGenerationError::GenerationBudgetDeferred
    );
    let retried = bake.build_atlas_from_slots_scheduled(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
        &scheduler,
        2,
    );
    assert_eq!(retried.report.compiled_atlas_build_count, 1);
    assert_eq!(retried.report.compiled_atlas_reuse_count, 0);
    assert!(!bake.glyphs.contains_key(&plan.slots[0].key));
    assert_eq!(scheduler.diagnostics(2).rejected_batch_count, 2);
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
    plan.slots[0].key.font_family = Some("Microsoft YaHei UI".into());
    plan.slots[0].key.language = Some("zh-Hans".into());
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
        font: Some(DEFAULT_FONT_ASSET.into()),
        font_family: Some("Microsoft YaHei UI".into()),
        language: Some("zh-hans".into()),
        font_weight: FontWeight::NORMAL.0,
        bake_params: SdfBakeParams::default(),
    };

    assert_eq!(glyph_index(font, &key, face, Some(face)), shaped_id);
}

#[test]
fn sdf_font_bake_does_not_match_the_old_rounded_rect_placeholder() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['A']);

    let atlas = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    let actual = slot_pixels_for_bake_page(&atlas, plan.atlas_size.x, 0, plan.slots[0].rect);
    let placeholder =
        old_rounded_rect_placeholder(plan.slots[0].rect.width, plan.slots[0].rect.height);
    assert_ne!(actual, placeholder);
    assert!(atlas.report.nonzero_pixel_count > 0);
}

fn old_rounded_rect_placeholder(width: u32, height: u32) -> Vec<u8> {
    const PADDING: f32 = 4.0;
    const SPREAD: f32 = 6.0;
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let half_width = (center_x - PADDING).max(1.0);
    let half_height = (center_y - PADDING).max(1.0);
    let mut pixels = Vec::with_capacity(width as usize * height as usize);

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 + 0.5 - center_x).abs() - half_width;
            let dy = (y as f32 + 0.5 - center_y).abs() - half_height;
            let outside_x = dx.max(0.0);
            let outside_y = dy.max(0.0);
            let outside_distance = (outside_x * outside_x + outside_y * outside_y).sqrt();
            let inside_distance = dx.max(dy).min(0.0);
            let signed_inside_distance = -(outside_distance + inside_distance);
            pixels.push(
                ((0.5 + signed_inside_distance / SPREAD).clamp(0.0, 1.0) * 255.0).round() as u8,
            );
        }
    }

    pixels
}
