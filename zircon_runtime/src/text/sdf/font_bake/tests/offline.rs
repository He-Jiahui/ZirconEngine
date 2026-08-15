use std::sync::atomic::{AtomicU64, Ordering};

use crate::asset::{AssetManager, AssetUri, ProjectAssetManager, ProjectManifest, ProjectPaths};
use crate::text::font::{load_text_font_source, FontDatabase};
use crate::text::sdf::{
    generate_distance_field_glyph, sdf_default_variation_hash, sdf_font_source_hash,
    sdf_offline_artifact_path, SdfBakeParams, SdfOfflineArtifact, SdfOfflineArtifactIdentity,
    SdfOfflineGlyph, SdfOfflineGlyphMetrics, SdfOfflinePage, SdfOfflineRect,
};
use zircon_runtime_interface::project::RelPath;

use super::super::{resolve_font_face, SdfFontBakeCache};

const TEXT_SDF_OFFLINE_WORK_DIRECTORY: &str = ".runtime_text_sdf_offline_work";

#[test]
fn text_sdf_offline_lookup_resolves_manifest_and_instance_once() {
    let source = include_str!("../offline_source.rs");

    assert!(!source.contains("resolve_font_face("));
    assert!(!source.contains("register_loaded_font_manifest("));
    assert!(source.contains("manifests: HashMap<String, Option<LoadedTextFontSource>>"));
    assert!(source.contains("artifacts: HashMap<SdfOfflineArtifactIdentity, Option<"));
    assert!(source.contains("glyph_bitmaps: HashMap<SdfOfflineGlyphKey, Arc<[u8]>>"));
    assert!(source.contains("MAX_RESIDENT_MANIFEST_COUNT"));
    assert!(source.contains("MAX_RESIDENT_ARTIFACT_BYTE_COUNT"));
    assert!(source.contains("MAX_RESIDENT_GLYPH_BITMAP_BYTE_COUNT"));
    assert!(source.contains("resident_artifact_byte_count"));
    assert!(source.contains("resident_glyph_bitmap_byte_count"));
}

#[test]
fn text_sdf_offline_negative_manifest_cache_has_a_hard_lru_limit() {
    let asset_manager = ProjectAssetManager::default();
    let mut bake = SdfFontBakeCache::new();

    for index in 0..129 {
        let font_ref = format!("res://fonts/missing-{index}.font.toml");
        assert!(bake
            .offline_source
            .load_manifest_for_test(&font_ref, &asset_manager)
            .is_none());
    }

    let report = bake.offline_source.report();
    assert_eq!(bake.offline_source.manifest_cache_len(), 128);
    assert_eq!(report.manifest_eviction_count, 1);
}

#[test]
fn text_sdf_font_bake_consumers_use_database_glyph_metadata_without_reparse() {
    let dynamic_source = include_str!("../distance_field.rs");
    let offline_source = include_str!("../offline_source.rs");

    assert!(!dynamic_source.contains("Face::parse"));
    assert!(!offline_source.contains("Face::parse"));
    assert!(dynamic_source.contains("face_glyph_id"));
    assert!(offline_source
        .contains("glyph_id_for_key(key, face_id, resolved_shaped_face, font_database)"));
}

#[test]
fn text_sdf_offline_glyph_hits_skip_dynamic_gen_and_miss_falls_back() {
    let fixture = OfflineFontProject::new();
    assert!(fixture.root.starts_with(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("zircon_runtime manifest must have a workspace parent")
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("text")
            .join(TEXT_SDF_OFFLINE_WORK_DIRECTORY)
    ));
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(fixture.root.to_string_lossy().as_ref())
        .expect("open font fixture project");
    let mut font_database = FontDatabase::with_default_fallbacks();
    let face_id = resolve_font_face(
        Some(OfflineFontProject::FONT_REF),
        &mut font_database,
        &asset_manager,
    )
    .expect("resolve fixture face");
    let face_bytes = font_database
        .standalone_face_bytes(face_id)
        .expect("standalone fixture face");
    let face = ttf_parser::Face::parse(face_bytes.as_ref(), 0).expect("parse fixture face");
    let glyph_id = face.glyph_index('A').expect("fixture A glyph").0;
    let params = SdfBakeParams::default();
    let generated = generate_distance_field_glyph(face_bytes.as_ref(), 0, glyph_id, params)
        .expect("generate offline A fixture");
    let manifest = load_text_font_source(OfflineFontProject::FONT_REF, Some(&asset_manager))
        .expect("load fixture font manifest");
    let identity = SdfOfflineArtifactIdentity {
        asset_guid: manifest.asset_uuid.expect("fixture asset UUID").to_string(),
        face_index: manifest.face_index,
        variation_hash: sdf_default_variation_hash(),
        source_hash: sdf_font_source_hash(face_bytes.as_ref()),
        params,
    };
    let artifact = SdfOfflineArtifact::new(
        identity.clone(),
        generated.size,
        vec![SdfOfflinePage {
            page_index: 0,
            pixels: generated.pixels.clone(),
        }],
        vec![SdfOfflineGlyph {
            glyph_id: u32::from(glyph_id),
            codepoint: 'A' as u32,
            page_index: 0,
            rect: SdfOfflineRect::new(0, 0, generated.size.x, generated.size.y),
            metrics: SdfOfflineGlyphMetrics {
                bitmap_left: generated.bitmap_left,
                bitmap_bottom: generated.bitmap_bottom,
                advance: generated.advance,
                ascent: generated.ascent,
            },
        }],
    )
    .expect("build offline fixture artifact");
    let project = asset_manager
        .current_project_manager()
        .expect("fixture project manager");
    let artifact_path = sdf_offline_artifact_path(project.paths().cache_root(), &identity);
    std::fs::create_dir_all(artifact_path.parent().expect("artifact parent")).unwrap();
    std::fs::write(&artifact_path, artifact.encode().unwrap()).unwrap();

    let mut bake = SdfFontBakeCache::new();
    let offline_plan = super::atlas_plan_for_asset('A', OfflineFontProject::FONT_REF);
    let offline = bake.build_atlas_from_slots(
        offline_plan.atlas_size,
        &offline_plan.slots,
        &mut font_database,
        &asset_manager,
    );
    let dynamic_plan = super::atlas_plan_for_asset('B', OfflineFontProject::FONT_REF);
    let dynamic = bake.build_atlas_from_slots(
        dynamic_plan.atlas_size,
        &dynamic_plan.slots,
        &mut font_database,
        &asset_manager,
    );

    assert_eq!(offline.report.offline_glyph_count, 1);
    assert_eq!(offline.report.dynamic_glyph_count, 0);
    assert!(offline.report.nonzero_pixel_count > 0);
    assert_eq!(offline.report.offline_manifest_parse_count, 1);
    assert_eq!(offline.report.offline_artifact_stat_count, 1);
    assert_eq!(offline.report.offline_artifact_read_count, 1);
    assert_eq!(offline.report.offline_artifact_decode_count, 1);
    assert_eq!(
        offline.report.offline_resident_artifact_byte_count,
        generated.pixels.len()
    );
    assert_eq!(
        offline.report.offline_resident_glyph_bitmap_byte_count,
        generated.pixels.len()
    );
    assert_eq!(offline.report.offline_pixel_copy_count, 1);
    assert_eq!(
        offline.report.offline_pixel_copy_byte_count,
        generated.pixels.len()
    );
    assert_eq!(offline.report.offline_manifest_eviction_count, 0);
    assert_eq!(offline.report.offline_artifact_eviction_count, 0);
    assert_eq!(offline.report.offline_glyph_bitmap_eviction_count, 0);
    assert_eq!(dynamic.report.offline_glyph_count, 0);
    assert_eq!(dynamic.report.dynamic_glyph_count, 1);
    assert!(dynamic.report.nonzero_pixel_count > 0);
    assert_eq!(dynamic.report.offline_manifest_parse_count, 0);
    assert_eq!(dynamic.report.offline_artifact_stat_count, 0);
    assert_eq!(dynamic.report.offline_artifact_read_count, 0);
    assert_eq!(dynamic.report.offline_artifact_decode_count, 0);
    assert_eq!(dynamic.report.offline_pixel_copy_count, 0);
    assert_eq!(bake.offline_source.manifest_cache_len(), 1);

    bake.clear_cached_glyph_entries();
    std::fs::remove_file(&artifact_path).unwrap();
    let reused = bake.build_atlas_from_slots(
        offline_plan.atlas_size,
        &offline_plan.slots,
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(reused.report.offline_glyph_count, 1);
    assert_eq!(reused.report.dynamic_glyph_count, 0);
    assert_eq!(reused.report.offline_artifact_stat_count, 0);
    assert_eq!(reused.report.offline_artifact_read_count, 0);
    assert_eq!(reused.report.offline_artifact_decode_count, 0);
    assert_eq!(reused.report.offline_pixel_copy_count, 0);

    let mut stale_variation = identity.clone();
    stale_variation.variation_hash = [0x11; 32];
    write_artifact_at_expected_path(&artifact_path, &artifact, stale_variation);
    let mut stale_variation_bake = SdfFontBakeCache::new();
    let stale_variation_result = stale_variation_bake.build_atlas_from_slots(
        offline_plan.atlas_size,
        &offline_plan.slots,
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(stale_variation_result.report.offline_glyph_count, 0);
    assert_eq!(stale_variation_result.report.dynamic_glyph_count, 1);
    assert_eq!(stale_variation_result.report.offline_artifact_stat_count, 1);
    assert_eq!(stale_variation_result.report.offline_artifact_read_count, 1);
    assert_eq!(
        stale_variation_result.report.offline_artifact_decode_count,
        1
    );
    assert_eq!(stale_variation_result.report.offline_pixel_copy_count, 0);

    stale_variation_bake.clear_cached_glyph_entries();
    let stale_variation_reused = stale_variation_bake.build_atlas_from_slots(
        offline_plan.atlas_size,
        &offline_plan.slots,
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(stale_variation_reused.report.offline_artifact_stat_count, 0);
    assert_eq!(stale_variation_reused.report.offline_artifact_read_count, 0);
    assert_eq!(
        stale_variation_reused.report.offline_artifact_decode_count,
        0
    );

    let mut stale_source = identity;
    stale_source.source_hash = [0x22; 32];
    write_artifact_at_expected_path(&artifact_path, &artifact, stale_source);
    let mut stale_source_bake = SdfFontBakeCache::new();
    let stale_source_result = stale_source_bake.build_atlas_from_slots(
        offline_plan.atlas_size,
        &offline_plan.slots,
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(stale_source_result.report.offline_glyph_count, 0);
    assert_eq!(stale_source_result.report.dynamic_glyph_count, 1);

    let late_manifest_ref = "res://fonts/late.font.toml";
    assert!(bake
        .offline_source
        .load_manifest_for_test(late_manifest_ref, &asset_manager)
        .is_none());
    std::fs::copy(
        fixture.font_root.join("offline.font.toml"),
        fixture.font_root.join("late.font.toml"),
    )
    .unwrap();
    asset_manager
        .open_project(fixture.root.to_string_lossy().as_ref())
        .expect("rescan fixture project after adding the late manifest");
    assert!(
        bake.offline_source
            .load_manifest_for_test(late_manifest_ref, &asset_manager)
            .is_none(),
        "a missing manifest must remain negatively cached for the current font generation"
    );
    assert_eq!(bake.offline_source.manifest_cache_len(), 2);

    let next_generation = bake.observed_font_generation.wrapping_add(1);
    bake.sync_font_generation(next_generation);
    assert_eq!(bake.offline_source.manifest_cache_len(), 0);
    assert!(bake
        .offline_source
        .load_manifest_for_test(late_manifest_ref, &asset_manager)
        .is_some());
}

fn write_artifact_at_expected_path(
    path: &std::path::Path,
    source: &SdfOfflineArtifact,
    identity: SdfOfflineArtifactIdentity,
) {
    let artifact = SdfOfflineArtifact::new(
        identity,
        source.page_size(),
        source.pages().to_vec(),
        source.glyphs().to_vec(),
    )
    .expect("build stale fixture artifact");
    std::fs::write(path, artifact.encode().unwrap()).unwrap();
}

struct OfflineFontProject {
    root: std::path::PathBuf,
    font_root: std::path::PathBuf,
}

impl OfflineFontProject {
    const FONT_REF: &'static str = "res://fonts/offline.font.toml";

    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("zircon_runtime manifest must have a workspace parent");
        let root = workspace_root
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("text")
            .join(TEXT_SDF_OFFLINE_WORK_DIRECTORY)
            .join(format!(
                "zircon-runtime-text-offline-sdf-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
        let paths = ProjectPaths::from_root(&root).expect("fixture paths");
        paths
            .ensure_layout(&[RelPath::project_assets()])
            .expect("fixture layout");
        ProjectManifest::new(
            "Offline SDF Fixture",
            AssetUri::parse("res://empty.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .expect("fixture manifest");
        let font_root = paths.asset_root(&RelPath::project_assets()).join("fonts");
        std::fs::create_dir_all(&font_root).unwrap();
        std::fs::copy(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/fonts/FiraSans-Regular.ttf"),
            font_root.join("FiraSans-Regular.ttf"),
        )
        .unwrap();
        std::fs::write(
            font_root.join("offline.font.toml"),
            "source = \"FiraSans-Regular.ttf\"\nfamily = \"Offline Fira Sans\"\n",
        )
        .unwrap();
        Self { root, font_root }
    }
}

impl Drop for OfflineFontProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
