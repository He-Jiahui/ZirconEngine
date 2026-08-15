use super::*;
use crate::text::font::register_font_handles;

#[test]
fn sdf_atlas_build_resolves_all_shaped_handles_in_one_batch() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::default();
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let registered = font_database
        .replace_font_source(
            "res://fonts/source-context-batch.ttf",
            &source,
            Some("SDF Source Context Batch"),
            0,
        )
        .expect("register source-context batch face");
    let face = registered.faces[0];
    let instance = font_database
        .default_instance_id(face)
        .expect("default instance");
    let generation = shared_font_database_generation();
    let (font_id, font_instance_id) = register_font_handles(Some(face), Some(instance), generation);
    let slots = "ABCD"
        .chars()
        .enumerate()
        .map(|(index, glyph)| SdfAtlasSlot {
            key: SdfAtlasGlyphKey {
                glyph,
                glyph_id: Some(40 + index as u32),
                font_id,
                font_instance_id,
                font: None,
                font_family: None,
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
        .collect::<Vec<_>>();
    let before = font_handle_registry_report();

    let _ = bake.build_atlas_from_slots(
        UVec2::new(256, 64),
        &slots,
        &mut font_database,
        &ProjectAssetManager::default(),
    );
    let after = font_handle_registry_report();

    assert_eq!(
        after.resolution_batch_count - before.resolution_batch_count,
        1
    );
    assert_eq!(bake.source_contexts.report().resident_context_count, 1);
}

#[test]
fn sdf_shaped_face_instance_mismatch_is_rejected_as_one_artifact() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let mut bake = SdfFontBakeCache::new();
    let mut font_database = FontDatabase::default();
    let font_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let first = font_database
        .replace_font_source(
            "res://fonts/mismatch-face.ttf",
            &font_root.join("FiraSans-Regular.ttf"),
            Some("SDF Mismatch Face"),
            0,
        )
        .expect("register mismatch face");
    let second = font_database
        .replace_font_source(
            "res://fonts/mismatch-instance.ttf",
            &font_root.join("FiraMono-subset.ttf"),
            Some("SDF Mismatch Instance"),
            0,
        )
        .expect("register mismatch instance face");
    let face = first.faces[0];
    let instance = font_database
        .default_instance_id(second.faces[0])
        .expect("mismatched instance");
    let (font_id, font_instance_id) = register_font_handles(
        Some(face),
        Some(instance),
        shared_font_database_generation(),
    );
    let key = SdfAtlasGlyphKey {
        glyph: 'A',
        glyph_id: Some(41),
        font_id,
        font_instance_id,
        font: None,
        font_family: None,
        language: None,
        font_weight: FontWeight::NORMAL.0,
        bake_params: SdfBakeParams::default(),
    };

    bake.prime_shaped_face_resolutions(std::slice::from_ref(&key), &font_database);

    assert_eq!(bake.shaped_face_resolutions.get(&key), Some(&None));
}
