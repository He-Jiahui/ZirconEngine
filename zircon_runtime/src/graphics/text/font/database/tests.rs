use std::path::Path;
use std::sync::Arc;

use glyphon::{fontdb, FontSystem};

use super::*;
use crate::asset::{FontAsset, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetRenderStrategy};
use crate::core::framework::render::{
    CompositeFontDescriptor, FontFaceDescriptor, FontFamilyName, FontScript, SubFontRange,
};
use crate::graphics::text::font::test_font_fixtures::{write_ttc_fixture, write_weight_fixture};

#[test]
fn text_font_database_query_best_match_weight_distance() {
    let mut database = FontDatabase::default();
    let regular = database
        .register_stored_face(
            FontFaceDescriptor::regular("Inter"),
            Arc::from([1_u8, 2, 3].as_slice()),
            None,
        )
        .unwrap();
    let mut bold_face = FontFaceDescriptor::regular("Inter");
    bold_face.weight = FontWeight::BOLD;
    let bold = database
        .register_stored_face(bold_face, Arc::from([4_u8, 5, 6].as_slice()), None)
        .unwrap();

    let query = FontQuery {
        families: vec![FontFamilyName::from("Inter")],
        weight: FontWeight::BOLD,
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    };

    assert_eq!(database.match_face(&query).unwrap().face, bold);
    assert_ne!(database.match_face(&query).unwrap().face, regular);
}

#[test]
fn text_font_face_shares_arc_bytes_across_backends() {
    let mut database = FontDatabase::default();
    let bytes: Arc<[u8]> = Arc::from([9_u8, 8, 7, 6].as_slice());
    let face = database
        .register_stored_face(
            FontFaceDescriptor::regular("Inter"),
            Arc::clone(&bytes),
            None,
        )
        .unwrap();

    let glyphon_bytes = database.face_bytes(face).unwrap();
    let sdf_bytes = database.face_bytes(face).unwrap();

    assert!(Arc::ptr_eq(&glyphon_bytes, &sdf_bytes));
    assert!(Arc::ptr_eq(&glyphon_bytes, &bytes));
}

#[test]
fn text_font_variations_hash_stable() {
    let mut database = FontDatabase::default();
    let face = database
        .register_stored_face(
            FontFaceDescriptor::regular("Inter"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .unwrap();
    let variations = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 650.0)]);
    let same = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 650.0)]);
    let different = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 700.0)]);

    assert_eq!(
        database.instance(face, &variations).unwrap(),
        database.instance(face, &same).unwrap()
    );
    assert_ne!(
        database.instance(face, &variations).unwrap(),
        database.instance(face, &different).unwrap()
    );
}

#[test]
fn text_font_database_registers_file_once_and_feeds_glyphon_fontdb() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraMono-subset.ttf");
    let mut database = FontDatabase::default();

    let first = database
        .register_font_file(&source_path, Some("Fira Mono"), 0)
        .unwrap();
    let second = database
        .register_font_file(&source_path, Some("Fira Mono"), 0)
        .unwrap();

    assert_eq!(first, second);
    assert!(database.face_bytes(first).unwrap().len() > 0);

    let mut font_system = FontSystem::new();
    database
        .load_face_into_font_system(first, &mut font_system)
        .unwrap();

    let families = [fontdb::Family::Name("Fira Mono")];
    let query = fontdb::Query {
        families: &families,
        ..fontdb::Query::default()
    };
    assert!(font_system.db_mut().query(&query).is_some());
}

#[test]
fn text_font_database_reads_file_weight_for_best_match() {
    let regular_path = write_weight_fixture("regular", 400);
    let bold_path = write_weight_fixture("bold", 700);
    let mut database = FontDatabase::default();

    let regular = database
        .register_font_file(&regular_path, Some("Fira Metadata Test"), 0)
        .unwrap();
    let bold = database
        .register_font_file(&bold_path, Some("Fira Metadata Test"), 0)
        .unwrap();
    let query = FontQuery {
        families: vec![FontFamilyName::from("Fira Metadata Test")],
        weight: FontWeight::BOLD,
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    };

    assert_eq!(database.match_face(&query).unwrap().face, bold);
    assert_ne!(database.match_face(&query).unwrap().face, regular);

    let _ = std::fs::remove_file(regular_path);
    let _ = std::fs::remove_file(bold_path);
}

#[test]
fn text_font_database_registers_ttc_faces_by_index() {
    let collection_path = write_ttc_fixture();
    let mut database = FontDatabase::default();

    let regular = database
        .register_font_file(&collection_path, Some("Fira TTC Test"), 0)
        .unwrap();
    let bold = database
        .register_font_file(&collection_path, Some("Fira TTC Test"), 1)
        .unwrap();
    let repeated_bold = database
        .register_font_file(&collection_path, Some("Fira TTC Test"), 1)
        .unwrap();
    let query = FontQuery {
        families: vec![FontFamilyName::from("Fira TTC Test")],
        weight: FontWeight::BOLD,
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    };

    assert_ne!(regular, bold);
    assert_eq!(bold, repeated_bold);
    assert_eq!(database.face_index(regular).unwrap(), 0);
    assert_eq!(database.face_index(bold).unwrap(), 1);
    assert_eq!(database.match_face(&query).unwrap().face, bold);

    let _ = std::fs::remove_file(collection_path);
}

#[test]
fn text_font_database_registers_font_asset_family_members_and_fallbacks() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraMono-subset.ttf");
    let mut database = FontDatabase::default();
    let asset = FontAsset {
        source: "FiraMono-subset.ttf".to_string(),
        family: Some("Primary Sans".to_string()),
        render_mode: None,
        face_index: 0,
        family_members: vec![
            FontAssetFamilyMember {
                family: "Primary Sans".to_string(),
                face_index: 0,
                weight: Some(400),
                width_class: Some(5),
                style: Some(FontAssetFaceStyle::Normal),
                variations: Vec::new(),
            },
            FontAssetFamilyMember {
                family: "Primary Sans".to_string(),
                face_index: 0,
                weight: Some(700),
                width_class: Some(3),
                style: Some(FontAssetFaceStyle::Italic),
                variations: Vec::new(),
            },
        ],
        variable_instances: Vec::new(),
        fallback_families: vec!["Fallback Sans".to_string()],
        render_strategy: FontAssetRenderStrategy::default(),
        metadata: None,
    };
    let fallback = database
        .register_stored_face(
            FontFaceDescriptor::regular("Fallback Sans"),
            Arc::from([7_u8, 8, 9].as_slice()),
            None,
        )
        .unwrap();

    let registered = database.register_font_asset(&asset, &source_path).unwrap();
    let registered_again = database.register_font_asset(&asset, &source_path).unwrap();
    let regular_query = FontQuery {
        families: vec![FontFamilyName::from("Primary Sans")],
        weight: FontWeight::NORMAL,
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    };
    let bold_query = FontQuery {
        families: vec![FontFamilyName::from("Primary Sans")],
        weight: FontWeight::BOLD,
        style: FontStyle::Italic,
        stretch: FontStretch::clamped(75),
    };
    let fallback_query = FontQuery::single_family("Missing Primary");

    assert_eq!(registered.len(), 2);
    assert_eq!(registered_again, registered);
    assert_ne!(registered[0], registered[1]);
    assert_eq!(
        database.match_face(&regular_query).unwrap().face,
        registered[0]
    );
    assert_eq!(
        database.match_face(&bold_query).unwrap().face,
        registered[1]
    );
    assert_eq!(database.match_face(&fallback_query).unwrap().face, fallback);
    assert!(database
        .fallback_families()
        .iter()
        .any(|family| family.as_str() == "Fallback Sans"));
}

#[test]
fn text_font_database_composite_candidates_prioritize_matching_subfont() {
    let mut database = FontDatabase::with_default_fallbacks();
    let latin = database
        .register_stored_face(
            FontFaceDescriptor::regular("Inter"),
            Arc::from([1_u8, 2, 3].as_slice()),
            None,
        )
        .unwrap();
    let cjk = database
        .register_stored_face(
            FontFaceDescriptor::regular("Noto Sans CJK SC"),
            Arc::from([4_u8, 5, 6].as_slice()),
            None,
        )
        .unwrap();
    let query = FontQuery::single_family("Inter");
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Inter"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Noto Sans CJK SC"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x4E00, 0x9FFF)],
        }],
    };

    let cjk_candidates = database.fallback_candidates('界', &query, Some(&composite));
    assert_eq!(cjk_candidates.first().copied(), Some(cjk));
    assert!(cjk_candidates.contains(&latin));

    let latin_candidates = database.fallback_candidates('A', &query, Some(&composite));
    assert_eq!(latin_candidates.first().copied(), Some(latin));
}

#[test]
fn text_font_database_resolves_fallback_face_for_codepoint() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let primary = database
        .register_font_file(&source_path, Some("Inter"), 0)
        .unwrap();
    let cjk = database
        .register_stored_face(
            FontFaceDescriptor::regular("Noto Sans CJK SC"),
            Arc::from([4_u8, 5, 6].as_slice()),
            None,
        )
        .unwrap();
    let query = FontQuery::single_family("Inter");
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Inter"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Noto Sans CJK SC"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x4E00, 0x9FFF)],
        }],
    };

    let cjk_face =
        database.resolve_fallback_face_for_codepoint(primary, '界', &query, Some(&composite));
    let latin_face =
        database.resolve_fallback_face_for_codepoint(primary, 'A', &query, Some(&composite));

    assert_eq!(cjk_face, cjk);
    assert_eq!(latin_face, primary);
}

#[test]
fn text_font_fallback_candidates_filter_known_cmap_coverage() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let latin = database
        .register_font_file(&source_path, Some("Mixed Coverage"), 0)
        .unwrap();
    let unknown = database
        .register_stored_face(
            FontFaceDescriptor::regular("Mixed Coverage"),
            Arc::from([1_u8, 2, 3].as_slice()),
            None,
        )
        .unwrap();
    let query = FontQuery::single_family("Mixed Coverage");

    let latin_candidates = database.fallback_candidates('A', &query, None);
    let cjk_candidates = database.fallback_candidates('界', &query, None);

    assert!(latin_candidates.contains(&latin));
    assert!(latin_candidates.contains(&unknown));
    assert!(!cjk_candidates.contains(&latin));
    assert!(cjk_candidates.contains(&unknown));
}
