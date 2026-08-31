use super::font_asset_cache::MAX_CACHED_FONT_ASSET_FACE_COUNT;
use super::*;
use crate::asset::ProjectAssetManager;
use crate::core::framework::text::{TextFontCollectionHandle, TextFontFaceHandle};
use crate::core::math::UVec2;
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasPageKey};
use crate::text::font::{font_handle_registry_report, shared_font_database_test_serial_guard};
use crate::text::sdf::{SdfGenerationSchedulerDiagnostics, SdfGenerationSchedulerOptions, SdfMode};
use std::path::PathBuf;

mod cache_generation;
mod handle_resolution;
mod offline;
mod owner_recovery;

const TEXT_SDF_MANIFEST_WORK_DIRECTORY: &str = ".runtime_text_sdf_manifest_work";
const TEST_FONT_COLLECTION: TextFontCollectionHandle = TextFontCollectionHandle::new(1);

#[derive(Default)]
struct SdfAtlasPlan {
    atlas_size: UVec2,
    slots: Vec<SdfAtlasSlot>,
}

struct CpuPreparationRun {
    text: String,
    resolved_advances: Option<Vec<f32>>,
    shaped_glyphs: Vec<Option<SdfShapedGlyphIdentity>>,
}

impl SdfTextRun for CpuPreparationRun {
    fn font(&self) -> Option<&str> {
        Some(DEFAULT_FONT_ASSET)
    }

    fn font_family(&self) -> Option<&str> {
        Some("Studio Mono")
    }

    fn language(&self) -> Option<&str> {
        None
    }

    fn font_weight(&self) -> u16 {
        FontWeight::NORMAL.0
    }

    fn font_size(&self) -> f32 {
        18.0
    }

    fn render_scalars(&self) -> Vec<char> {
        self.text.chars().collect()
    }

    fn resolved_glyph_advances(&self) -> Option<Vec<f32>> {
        self.resolved_advances.clone()
    }

    fn shaped_glyph(&self, glyph_index: usize) -> Option<SdfShapedGlyphIdentity> {
        self.shaped_glyphs.get(glyph_index).copied().flatten()
    }
}

#[test]
fn sdf_cpu_preparation_batches_metrics_advances_and_invisible_controls() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let generation = bake.observed_font_generation;
    let run = CpuPreparationRun {
        text: "A\u{200b} ".to_string(),
        resolved_advances: None,
        shaped_glyphs: Vec::new(),
    };

    let prepared = bake.prepare_run_cpu(&run, &mut font_database, &asset_manager);

    assert_eq!(prepared.glyph_metrics.len(), 3);
    assert_eq!(prepared.glyph_advances.len(), 3);
    assert!(prepared.glyph_advances[0] > 0.0);
    assert_eq!(prepared.glyph_metrics[1], SdfGlyphMetrics::default());
    assert_eq!(prepared.glyph_advances[1], 0.0);
    assert!(prepared.glyph_advances[2] > 0.0);
}

#[test]
fn sdf_cpu_preparation_preserves_canonical_resolved_advances() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let run = CpuPreparationRun {
        text: "AB".to_string(),
        resolved_advances: Some(vec![11.0, 13.0]),
        shaped_glyphs: Vec::new(),
    };

    let prepared = bake.prepare_run_cpu(&run, &mut font_database, &asset_manager);

    assert_eq!(prepared.glyph_advances, vec![11.0, 13.0]);
    assert_eq!(prepared.glyph_metrics.len(), 2);
}

#[test]
fn sdf_font_resolver_reuses_registered_asset_owner_without_manifest_io() {
    let asset_ref = "res://fonts/does-not-exist.font.toml";
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut font_database = FontDatabase::default();
    let registered = font_database
        .replace_font_source(asset_ref, &source, Some("Cached SDF Face"), 0)
        .expect("register cached SDF face");

    let resolved = resolve_font_face(
        Some(asset_ref),
        &mut font_database,
        &ProjectAssetManager::default(),
    );

    assert_eq!(resolved, Ok(registered.faces[0]));
    assert_eq!(font_database.face_count(), 1);
}

#[test]
fn sdf_default_font_resolver_uses_runtime_baseline_without_manifest_io() {
    let mut font_database = FontDatabase::default();
    let baseline = font_database
        .register_test_face(
            crate::text::FontFaceDescriptor::regular("Runtime Default"),
            Arc::from([1_u8].as_slice()),
        )
        .expect("register runtime default fixture");
    assert!(font_database.set_runtime_default_primary_face(baseline));

    let resolved = resolve_font_face(
        Some(DEFAULT_FONT_ASSET),
        &mut font_database,
        &ProjectAssetManager::default(),
    );

    assert_eq!(resolved, Ok(baseline));
    assert_eq!(font_database.face_count(), 1);
    assert_eq!(
        font_database.font_asset_primary_face(DEFAULT_FONT_ASSET),
        None
    );
}

#[test]
fn sdf_runtime_font_lookup_does_not_admit_an_unregistered_asset() {
    let mut bake = SdfFontBakeCache::new();
    let font_database = FontDatabase::with_default_fallbacks();
    let face_count = font_database.face_count();

    let resolved = bake.font_asset_faces.resolve(
        bake.observed_font_generation,
        Some("res://fonts/unadmitted-runtime-font.font.toml"),
        &font_database,
    );

    assert_eq!(resolved, None);
    assert_eq!(font_database.face_count(), face_count);
    assert_eq!(
        bake.font_asset_faces
            .report()
            .resident_no_registered_faces_count,
        1
    );
}

#[test]
fn sdf_cpu_preparation_caches_shaped_metrics_across_frames() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let face = TextFontFaceHandle::new(TEST_FONT_COLLECTION, 7, 11);
    let run = CpuPreparationRun {
        text: "AB".to_string(),
        resolved_advances: None,
        shaped_glyphs: vec![
            Some(SdfShapedGlyphIdentity {
                glyph_id: 41,
                font_id: Some(face),
                font_instance_id: None,
            }),
            Some(SdfShapedGlyphIdentity {
                glyph_id: 42,
                font_id: Some(face),
                font_instance_id: None,
            }),
        ],
    };

    let first = bake.prepare_run_cpu(&run, &mut font_database, &asset_manager);
    let cached_after_first = bake.measured_glyphs.len();
    let second = bake.prepare_run_cpu(&run, &mut font_database, &asset_manager);

    assert_eq!(first.glyph_metrics, second.glyph_metrics);
    assert_eq!(cached_after_first, 2);
    assert_eq!(bake.measured_glyphs.len(), cached_after_first);
    assert_eq!(bake.face_resolutions.len(), cached_after_first);
    assert_eq!(bake.font_asset_faces.len(), 1);
    assert!(
        bake.measured_glyphs
            .keys()
            .all(|key| { key.font_id == Some(face) && key.glyph_id.is_some() })
    );
    assert!(
        bake.face_resolutions
            .keys()
            .all(|key| { key.font_id == Some(face) && key.glyph_id.is_some() })
    );
}

#[test]
fn sdf_font_asset_face_cache_evicts_oldest_asset_at_its_hard_limit() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    for index in 0..=MAX_CACHED_FONT_ASSET_FACE_COUNT {
        let asset = format!("res://fonts/missing-{index}.font.toml");
        let _ = bake.resolve_font_asset_face_cached(Some(&asset), &mut font_database);
    }

    assert_eq!(
        bake.font_asset_faces.len(),
        MAX_CACHED_FONT_ASSET_FACE_COUNT
    );
    assert!(
        !bake
            .font_asset_faces
            .contains(generation, "res://fonts/missing-0.font.toml")
    );
    assert!(bake.font_asset_faces.contains(
        generation,
        &format!(
            "res://fonts/missing-{}.font.toml",
            MAX_CACHED_FONT_ASSET_FACE_COUNT
        )
    ));
    assert_eq!(
        bake.font_asset_faces.report().resident_error_count,
        MAX_CACHED_FONT_ASSET_FACE_COUNT
    );
}

#[test]
fn sdf_cpu_preparation_resolves_all_shaped_handles_in_one_batch() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let stale_generation = shared_font_database_generation().saturating_add(1);
    let handle = TextFontFaceHandle::new(TEST_FONT_COLLECTION, 7, stale_generation);
    let run = CpuPreparationRun {
        text: "ABCD".to_string(),
        resolved_advances: None,
        shaped_glyphs: (0..4)
            .map(|index| {
                Some(SdfShapedGlyphIdentity {
                    glyph_id: 40 + index,
                    font_id: Some(handle),
                    font_instance_id: None,
                })
            })
            .collect(),
    };
    let before = font_handle_registry_report();

    let _ = bake.prepare_run_cpu(&run, &mut font_database, &asset_manager);
    let after = font_handle_registry_report();

    assert_eq!(
        after.resolution_batch_count - before.resolution_batch_count,
        1
    );
}

#[test]
fn sdf_face_invalidation_discards_all_face_derived_caches() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let run = CpuPreparationRun {
        text: "A".to_string(),
        resolved_advances: None,
        shaped_glyphs: Vec::new(),
    };

    let _ = bake.prepare_run_cpu(&run, &mut font_database, &asset_manager);
    assert!(!bake.measured_glyphs.is_empty());
    assert!(!bake.face_resolutions.is_empty());
    assert!(!bake.font_asset_faces.is_empty());

    bake.invalidate_faces();

    assert!(bake.fonts.is_empty());
    assert!(bake.glyphs.is_empty());
    assert!(bake.measured_glyphs.is_empty());
    assert!(bake.face_resolutions.is_empty());
    assert!(bake.font_asset_faces.is_empty());
}

#[test]
fn sdf_generation_rollover_discards_face_derived_caches_before_lookup() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let run = CpuPreparationRun {
        text: "A".to_string(),
        resolved_advances: None,
        shaped_glyphs: Vec::new(),
    };

    let _ = bake.prepare_run_cpu(&run, &mut font_database, &asset_manager);
    assert!(!bake.measured_glyphs.is_empty());
    assert!(!bake.face_resolutions.is_empty());
    assert!(!bake.font_asset_faces.is_empty());

    let next_generation = bake.observed_font_generation.saturating_add(1);
    bake.sync_font_generation(next_generation);

    assert_eq!(bake.observed_font_generation, next_generation);
    assert!(bake.fonts.is_empty());
    assert!(bake.glyphs.is_empty());
    assert!(bake.measured_glyphs.is_empty());
    assert!(bake.face_resolutions.is_empty());
    assert!(bake.font_asset_faces.is_empty());
}

#[test]
fn stale_shaped_face_handle_cannot_reuse_its_glyph_id_on_a_fallback_face() {
    let stale_generation = shared_font_database_generation().saturating_add(1);
    let key = SdfAtlasGlyphKey {
        glyph: 'A',
        glyph_id: Some(777),
        font_id: Some(TextFontFaceHandle::new(
            TEST_FONT_COLLECTION,
            0,
            stale_generation,
        )),
        font_instance_id: None,
        font: Some(DEFAULT_FONT_ASSET.into()),
        font_family: Some("Studio Mono".into()),
        language: None,
        font_weight: FontWeight::NORMAL.0,
        bake_params: SdfBakeParams::default(),
    };

    assert_eq!(shaped_glyph_id_for_face(&key, FontFaceId(1), None), None);
}

#[test]
fn sdf_scalar_slot_policy_is_owned_by_text() {
    for scalar in ['\u{061C}', '\u{200B}', '\u{FE0F}', '\u{E0100}'] {
        assert!(sdf_scalar_is_invisible_format(scalar));
        assert!(!sdf_scalar_requires_atlas_slot(scalar));
    }
    assert!(sdf_scalar_requires_atlas_slot('中'));
    assert!(!sdf_scalar_requires_atlas_slot(' '));
}

#[test]
fn sdf_font_bake_produces_distinct_ascii_glyph_patterns() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['A', 'I', 'O']);

    let atlas = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    let a = slot_pixels_for_bake_page(&atlas, plan.atlas_size.x, 0, plan.slots[0].rect);
    let i = slot_pixels_for_bake_page(&atlas, plan.atlas_size.x, 0, plan.slots[1].rect);
    let o = slot_pixels_for_bake_page(&atlas, plan.atlas_size.x, 0, plan.slots[2].rect);
    assert_ne!(a, i);
    assert_ne!(a, o);
    assert_ne!(i, o);
    assert_eq!(atlas.report.slot_count, 3);
    assert_eq!(atlas.report.visible_glyph_count, 3);
    assert_eq!(atlas.report.empty_glyph_count, 0);
    assert_eq!(
        atlas.report.atlas_byte_len,
        (plan.atlas_size.x * plan.atlas_size.y) as usize
    );
    assert!(atlas.report.nonzero_pixel_count > 0);
    assert!(atlas.report.loaded_font_count >= 1);
    assert_eq!(
        atlas.report.loaded_font_count,
        atlas.report.resident_font_count
    );
}

#[test]
fn sdf_font_bake_writes_page_indexed_slots_into_matching_layers() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_page_glyphs(&[('A', 0), ('B', 1)]);

    let atlas = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    let page_byte_len = (plan.atlas_size.x * plan.atlas_size.y) as usize;
    let a = slot_pixels_for_bake_page(&atlas, plan.atlas_size.x, 0, plan.slots[0].rect);
    let b = slot_pixels_for_bake_page(&atlas, plan.atlas_size.x, 1, plan.slots[1].rect);
    assert_eq!(atlas.pages.len(), 2);
    assert_eq!(atlas.report.atlas_byte_len, page_byte_len * 2);
    assert!(a.iter().any(|pixel| *pixel != 0));
    assert!(b.iter().any(|pixel| *pixel != 0));
    assert_ne!(a, b);
}

#[test]
fn sdf_font_bake_measures_whitespace_without_atlas_bitmap() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();

    let metrics = bake.measure_glyph(
        ' ',
        Some(DEFAULT_FONT_ASSET),
        Some("Studio Mono"),
        None,
        FontWeight::NORMAL.0,
        18.0,
        &mut font_database,
        &asset_manager,
    );

    assert!(metrics.advance > 0.0);
    assert_eq!(metrics.bitmap_width, 0);
    assert_eq!(metrics.bitmap_height, 0);
}

#[test]
fn sdf_font_bake_handles_missing_glyph_with_stable_empty_fallback() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = atlas_plan_for_glyphs(&['\u{10ffff}']);

    let metrics = bake.measure_glyph(
        '\u{10ffff}',
        Some(DEFAULT_FONT_ASSET),
        Some("Studio Mono"),
        None,
        FontWeight::NORMAL.0,
        18.0,
        &mut font_database,
        &asset_manager,
    );

    assert!(metrics.advance > 0.0);

    let atlas = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(atlas.glyphs.len(), 1);
    assert!(atlas.glyphs[0].metrics.advance > 0.0);
    assert_eq!(atlas.pages.len(), 1);
    assert_eq!(atlas.report.slot_count, 1);
    assert_eq!(
        atlas.report.atlas_byte_len,
        (plan.atlas_size.x * plan.atlas_size.y) as usize
    );
}

#[test]
fn sdf_font_query_for_key_preserves_font_weight() {
    let query = font_query_for_key(&SdfAtlasGlyphKey {
        glyph: 'A',
        glyph_id: None,
        font_id: None,
        font_instance_id: None,
        font: Some(DEFAULT_FONT_ASSET.into()),
        font_family: Some("Studio Mono".into()),
        language: None,
        font_weight: 650,
        bake_params: SdfBakeParams::default(),
    });

    assert_eq!(query.weight, FontWeight::clamped(650));
}

#[test]
fn sdf_font_bake_falls_back_when_fontsdf_cannot_open_requested_face_index() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let manifest = write_face_index_manifest(1);
    assert!(
        manifest.path().starts_with(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("zircon_runtime manifest must have a workspace parent")
                .join("docs")
                .join("tests")
                .join("runtime")
                .join("text")
                .join(TEXT_SDF_MANIFEST_WORK_DIRECTORY)
        )
    );
    let plan = atlas_plan_for_asset('A', manifest.path().to_string_lossy().as_ref());

    let atlas = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    assert_eq!(atlas.report.slot_count, 1);
    assert_eq!(atlas.report.visible_glyph_count, 1);
    assert!(atlas.report.nonzero_pixel_count > 0);
}

#[test]
fn sdf_font_bake_report_handles_empty_atlas_plan() {
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::with_default_fallbacks();
    let asset_manager = ProjectAssetManager::default();
    let plan = SdfAtlasPlan {
        atlas_size: UVec2::new(1, 1),
        slots: Vec::new(),
    };

    let atlas = bake.build_atlas_from_slots(
        plan.atlas_size,
        &plan.slots,
        &mut font_database,
        &asset_manager,
    );

    assert!(atlas.pages.is_empty());
    assert_eq!(
        atlas.report,
        SdfAtlasBakeReport {
            slot_count: 0,
            visible_glyph_count: 0,
            empty_glyph_count: 0,
            atlas_byte_len: 0,
            nonzero_pixel_count: 0,
            resident_font_count: 0,
            loaded_font_count: 0,
            generation_failure_count: 0,
            resident_font_asset_error_count: 0,
            resident_font_asset_no_registered_faces_count: 0,
            r8_byte_len: 0,
            rgba_byte_len: 0,
            offline_glyph_count: 0,
            dynamic_glyph_count: 0,
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
            resident_baked_glyph_count: 0,
            resident_baked_glyph_byte_count: 0,
            baked_glyph_eviction_count: 0,
            oldest_baked_glyph_idle_access_count: 0,
            resident_source_context_count: 0,
            resident_source_byte_count: 0,
            source_context_created_count: 0,
            source_context_eviction_count: 0,
            oldest_source_context_idle_access_count: 0,
            source_hash_count: 0,
            face_parse_count: 0,
            generation_batch_count: 0,
            generation_requested_glyph_count: 0,
            generation_unique_glyph_count: 0,
            generation_duplicate_glyph_count: 0,
            bitmap_clone_byte_count: 0,
            resident_atlas_page_count: 0,
            atlas_page_alloc_count: 0,
            atlas_page_zero_byte_count: 0,
            atlas_page_clear_count: 0,
            atlas_page_clear_byte_count: 0,
            atlas_page_write_count: 0,
            atlas_page_write_byte_count: 0,
            atlas_page_reused_slot_count: 0,
            atlas_full_page_scan_byte_count: 0,
            compiled_atlas_build_count: 1,
            compiled_atlas_reuse_count: 0,
            generation_scheduler: SdfGenerationSchedulerDiagnostics::default(),
        }
    );
}

fn atlas_plan_for_glyphs(glyphs: &[char]) -> SdfAtlasPlan {
    let slots = glyphs
        .iter()
        .enumerate()
        .map(|(index, glyph)| SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph: *glyph,
                glyph_id: None,
                font_id: None,
                font_instance_id: None,
                font: Some(DEFAULT_FONT_ASSET.into()),
                font_family: Some("Studio Mono".into()),
                language: None,
                font_weight: FontWeight::NORMAL.0,
                bake_params: SdfBakeParams::default(),
            },
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0),
            rect: SdfAtlasRect {
                x: index as u32 * 64,
                y: 0,
                width: 64,
                height: 64,
            },
        })
        .collect();
    SdfAtlasPlan {
        atlas_size: UVec2::new(256, 256),
        slots,
    }
}

fn atlas_plan_for_page_glyphs(glyphs: &[(char, u32)]) -> SdfAtlasPlan {
    let slots = glyphs
        .iter()
        .map(|(glyph, page_index)| SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph: *glyph,
                glyph_id: None,
                font_id: None,
                font_instance_id: None,
                font: Some(DEFAULT_FONT_ASSET.into()),
                font_family: Some("Studio Mono".into()),
                language: None,
                font_weight: FontWeight::NORMAL.0,
                bake_params: SdfBakeParams::default(),
            },
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, *page_index),
            rect: SdfAtlasRect {
                x: 0,
                y: 0,
                width: 64,
                height: 64,
            },
        })
        .collect();
    SdfAtlasPlan {
        atlas_size: UVec2::new(256, 256),
        slots,
    }
}

fn atlas_plan_for_mixed_distance_fields() -> SdfAtlasPlan {
    let mut sdf = SdfBakeParams::default();
    sdf.mode = SdfMode::Sdf;
    let mut msdf = sdf;
    msdf.mode = SdfMode::Msdf;
    let mut mtsdf = sdf;
    mtsdf.mode = SdfMode::Mtsdf;
    let glyphs = [
        ('A', sdf, GlyphAtlasFormat::Sdf, 0),
        ('M', msdf, GlyphAtlasFormat::Msdf, 0),
        ('W', mtsdf, GlyphAtlasFormat::Msdf, 64),
        ('\u{10ffff}', msdf, GlyphAtlasFormat::Msdf, 128),
    ];
    let slots = glyphs
        .into_iter()
        .map(|(glyph, bake_params, format, x)| SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph,
                glyph_id: None,
                font_id: None,
                font_instance_id: None,
                font: Some(DEFAULT_FONT_ASSET.into()),
                font_family: Some("Studio Mono".into()),
                language: None,
                font_weight: FontWeight::NORMAL.0,
                bake_params,
            },
            page_key: GlyphAtlasPageKey::new(format, 0),
            rect: SdfAtlasRect {
                x,
                y: 0,
                width: 64,
                height: 64,
            },
        })
        .collect();
    SdfAtlasPlan {
        atlas_size: UVec2::new(256, 256),
        slots,
    }
}

fn atlas_plan_for_asset(glyph: char, asset_ref: &str) -> SdfAtlasPlan {
    SdfAtlasPlan {
        atlas_size: UVec2::new(64, 64),
        slots: vec![SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph,
                glyph_id: None,
                font_id: None,
                font_instance_id: None,
                font: Some(asset_ref.into()),
                font_family: Some("Fira Unsupported Face".into()),
                language: None,
                font_weight: FontWeight::NORMAL.0,
                bake_params: SdfBakeParams::default(),
            },
            page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0),
            rect: SdfAtlasRect {
                x: 0,
                y: 0,
                width: 64,
                height: 64,
            },
        }],
    }
}

struct TemporaryFontManifest {
    manifest: PathBuf,
    source: PathBuf,
}

impl TemporaryFontManifest {
    fn path(&self) -> &std::path::Path {
        &self.manifest
    }
}

impl Drop for TemporaryFontManifest {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.source);
        let _ = std::fs::remove_file(&self.manifest);
    }
}

fn write_face_index_manifest(face_index: u32) -> TemporaryFontManifest {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest must have a workspace parent")
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("text")
        .join(TEXT_SDF_MANIFEST_WORK_DIRECTORY);
    std::fs::create_dir_all(&root).expect("workspace SDF manifest directory should exist");
    let stem = format!("zircon-runtime-text-sdf-face-index-{}", std::process::id());
    let manifest = root.join(format!("{stem}.font.toml"));
    let source_name = format!("{stem}.ttf");
    let source = root.join(&source_name);
    std::fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("fonts")
            .join("FiraSans-Regular.ttf"),
        &source,
    )
    .unwrap();
    std::fs::write(
        &manifest,
        format!(
            "source = \"{source_name}\"\nfamily = \"Fira Unsupported Face\"\nface_index = {face_index}\n"
        ),
    )
    .unwrap();
    TemporaryFontManifest { manifest, source }
}

fn slot_pixels_for_bake_page(
    bake: &SdfAtlasBake,
    atlas_width: u32,
    page_index: usize,
    rect: SdfAtlasRect,
) -> Vec<u8> {
    let page = &bake.pages[page_index];
    let bytes_per_pixel = page.page_key.format.storage_format().bytes_per_pixel() as usize;
    let mut slot = Vec::with_capacity(rect.width as usize * rect.height as usize * bytes_per_pixel);
    for y in rect.y..rect.y + rect.height {
        let start = (y as usize * atlas_width as usize + rect.x as usize) * bytes_per_pixel;
        let end = start + rect.width as usize * bytes_per_pixel;
        slot.extend_from_slice(&page.pixels[start..end]);
    }
    slot
}

fn baked_glyph_rect(slot: SdfAtlasRect, glyph: &SdfBakedGlyph) -> SdfAtlasRect {
    SdfAtlasRect {
        width: glyph.metrics.bitmap_width.min(slot.width),
        height: glyph.metrics.bitmap_height.min(slot.height),
        ..slot
    }
}
