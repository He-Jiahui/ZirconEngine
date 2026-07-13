use std::sync::atomic::{AtomicU64, Ordering};

use crate::asset::{AssetManager, AssetUri, ProjectAssetManager, ProjectManifest, ProjectPaths};
use crate::graphics::scene::scene_renderer::ui::font_asset::load_ui_font_manifest_with_asset_manager;
use crate::graphics::text::font::FontDatabase;
use crate::graphics::text::sdf::{
    generate_distance_field_glyph, sdf_default_variation_hash, sdf_font_source_hash,
    sdf_offline_artifact_path, SdfBakeParams, SdfOfflineArtifact, SdfOfflineArtifactIdentity,
    SdfOfflineGlyph, SdfOfflineGlyphMetrics, SdfOfflinePage, SdfOfflineRect,
};
use zircon_runtime_interface::project::RelPath;

use super::super::{resolve_font_face, SdfFontBakeCache};

#[test]
fn text_sdf_offline_glyph_hits_skip_dynamic_gen_and_miss_falls_back() {
    let fixture = OfflineFontProject::new();
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
    let manifest = load_ui_font_manifest_with_asset_manager(
        OfflineFontProject::FONT_REF,
        Some(&asset_manager),
    )
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
    let offline = bake.build_atlas(
        &super::atlas_plan_for_asset('A', OfflineFontProject::FONT_REF),
        &mut font_database,
        &asset_manager,
    );
    let dynamic = bake.build_atlas(
        &super::atlas_plan_for_asset('B', OfflineFontProject::FONT_REF),
        &mut font_database,
        &asset_manager,
    );

    assert_eq!(offline.report.offline_glyph_count, 1);
    assert_eq!(offline.report.dynamic_glyph_count, 0);
    assert!(offline.report.nonzero_pixel_count > 0);
    assert_eq!(dynamic.report.offline_glyph_count, 0);
    assert_eq!(dynamic.report.dynamic_glyph_count, 1);
    assert!(dynamic.report.nonzero_pixel_count > 0);

    bake.glyphs.clear();
    std::fs::remove_file(&artifact_path).unwrap();
    let reused = bake.build_atlas(
        &super::atlas_plan_for_asset('A', OfflineFontProject::FONT_REF),
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(reused.report.offline_glyph_count, 1);
    assert_eq!(reused.report.dynamic_glyph_count, 0);

    let mut stale_variation = identity.clone();
    stale_variation.variation_hash = [0x11; 32];
    write_artifact_at_expected_path(&artifact_path, &artifact, stale_variation);
    let mut stale_variation_bake = SdfFontBakeCache::new();
    let stale_variation_result = stale_variation_bake.build_atlas(
        &super::atlas_plan_for_asset('A', OfflineFontProject::FONT_REF),
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(stale_variation_result.report.offline_glyph_count, 0);
    assert_eq!(stale_variation_result.report.dynamic_glyph_count, 1);

    let mut stale_source = identity;
    stale_source.source_hash = [0x22; 32];
    write_artifact_at_expected_path(&artifact_path, &artifact, stale_source);
    let mut stale_source_bake = SdfFontBakeCache::new();
    let stale_source_result = stale_source_bake.build_atlas(
        &super::atlas_plan_for_asset('A', OfflineFontProject::FONT_REF),
        &mut font_database,
        &asset_manager,
    );
    assert_eq!(stale_source_result.report.offline_glyph_count, 0);
    assert_eq!(stale_source_result.report.dynamic_glyph_count, 1);
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
}

impl OfflineFontProject {
    const FONT_REF: &'static str = "res://fonts/offline.font.toml";

    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
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
        Self { root }
    }
}

impl Drop for OfflineFontProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
