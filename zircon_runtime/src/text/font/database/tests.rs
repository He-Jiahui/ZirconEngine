use std::path::Path;
use std::sync::Arc;

use glyphon::{fontdb, FontSystem};
use ttf2woff2::{encode, BrotliQuality};

use super::*;
use crate::asset::{FontAsset, FontAssetFaceStyle, FontAssetFamilyMember, FontAssetRenderStrategy};
use crate::text::font::test_font_fixtures::{write_ttc_fixture, write_weight_fixture};
use crate::text::{
    CompositeFontDescriptor, FontCultureTag, FontFaceDescriptor, FontFamilyName, FontScript,
    SubFontRange,
};

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
fn text_font_database_match_cache_invalidates_when_a_better_face_registers() {
    let mut database = FontDatabase::default();
    let regular = database
        .register_stored_face(
            FontFaceDescriptor::regular("Layout Cache"),
            Arc::from([1_u8, 2, 3].as_slice()),
            None,
        )
        .expect("regular layout-cache face should register");
    let query = FontQuery {
        families: vec![FontFamilyName::from("Layout Cache")],
        weight: FontWeight::BOLD,
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    };

    assert_eq!(database.match_face(&query).unwrap().face, regular);

    let mut bold_descriptor = FontFaceDescriptor::regular("Layout Cache");
    bold_descriptor.weight = FontWeight::BOLD;
    let bold = database
        .register_stored_face(bold_descriptor, Arc::from([4_u8, 5, 6].as_slice()), None)
        .expect("better layout-cache face should register");

    assert_eq!(
        database.match_face(&query).unwrap().face,
        bold,
        "a cached primary match must not survive database mutation"
    );
}

#[test]
fn text_font_database_effective_instance_cache_reuses_weight_resolution_and_invalidates() {
    let mut database = FontDatabase::default();
    let face = database
        .register_stored_face(
            FontFaceDescriptor::regular("Layout Instance Cache"),
            Arc::from([1_u8, 2, 3].as_slice()),
            None,
        )
        .expect("layout instance-cache face should register");

    let first = database
        .effective_instance_id(face, 600)
        .expect("first effective instance should resolve");
    let second = database
        .effective_instance_id(face, 600)
        .expect("cached effective instance should resolve");
    assert_eq!(first, second);
    assert_eq!(database.effective_instance_cache_len(), 1);

    database
        .register_stored_face(
            FontFaceDescriptor::regular("Layout Instance Cache Mutation"),
            Arc::from([4_u8, 5, 6].as_slice()),
            None,
        )
        .expect("database mutation should register");
    assert_eq!(
        database.effective_instance_cache_len(),
        0,
        "stored-face mutation must not leave stale instance variations cached"
    );
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
fn text_font_database_system_font_policy_defaults_to_disabled() {
    let mut database = FontDatabase::default();

    assert_eq!(
        database.apply_system_font_policy(SystemFontPolicy::default()),
        0
    );
    assert!(database.faces.is_empty());
}

#[test]
fn text_font_database_defers_discovered_system_coverage_until_the_face_is_used() {
    if !cfg!(target_os = "windows") {
        return;
    }
    let mut database = FontDatabase::with_default_fallbacks();
    assert!(
        database.apply_system_font_policy(SystemFontPolicy::Discover) > 0,
        "the Windows text acceptance environment must expose the system font catalog"
    );

    let face = database
        .match_face(&FontQuery::single_family("Microsoft YaHei UI"))
        .expect("Windows CJK system font should be registered")
        .face;
    assert!(
        !database.coverage_is_initialized(face),
        "system discovery must not scan every cmap before the first UI text layout"
    );

    assert!(database.face_covers_codepoint(face, '中'));
    assert!(
        database.coverage_is_initialized(face),
        "the requested font coverage should be cached after the first lookup"
    );
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
fn text_font_variations_hash_normalizes_coordinate_order() {
    let mut database = FontDatabase::default();
    let face = database
        .register_stored_face(
            FontFaceDescriptor::regular("Inter Variable"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .unwrap();
    let forward = VariationCoords(vec![
        (u32::from_be_bytes(*b"wght"), 650.0),
        (u32::from_be_bytes(*b"wdth"), 90.0),
    ]);
    let reversed = VariationCoords(vec![
        (u32::from_be_bytes(*b"wdth"), 90.0),
        (u32::from_be_bytes(*b"wght"), 650.0),
    ]);

    assert_eq!(
        database.instance(face, &forward).unwrap(),
        database.instance(face, &reversed).unwrap()
    );
}

#[test]
fn text_font_database_registers_descriptor_variations_as_default_instance() {
    let mut database = FontDatabase::default();
    let mut descriptor = FontFaceDescriptor::regular("Inter Variable");
    descriptor.variations = VariationCoords(vec![
        (u32::from_be_bytes(*b"wght"), 650.0),
        (u32::from_be_bytes(*b"wdth"), 90.0),
    ]);
    let face = database
        .register_stored_face(descriptor, Arc::from([1_u8].as_slice()), None)
        .unwrap();

    let instance_id = database.default_instance_id(face).unwrap();
    let instance = database.font_instance(instance_id).unwrap();

    assert_eq!(instance.face, face);
    assert_eq!(
        instance.variations,
        VariationCoords(vec![
            (u32::from_be_bytes(*b"wdth"), 90.0),
            (u32::from_be_bytes(*b"wght"), 650.0),
        ])
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_effective_instance_id_tracks_real_weight_axis() {
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(
            Path::new(r"C:\Windows\Fonts\bahnschrift.ttf"),
            Some("Bahnschrift Variable Test"),
            0,
        )
        .expect("register Windows variable font");

    assert_ne!(
        database.effective_instance_id(face, 300).unwrap(),
        database.effective_instance_id(face, 700).unwrap()
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_effective_variations_merge_descriptor_axes_and_ui_weight() {
    let source = Path::new(r"C:\Windows\Fonts\bahnschrift.ttf");
    let bytes = std::fs::read(source).expect("Windows variable-font fixture");
    let parsed = ttf_parser::Face::parse(&bytes, 0).expect("parse Bahnschrift");
    let width = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let weight = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wght"))
        .expect("Bahnschrift weight axis");
    let mut descriptor = FontFaceDescriptor::regular("Bahnschrift Descriptor Variable Test");
    descriptor.variations = VariationCoords(vec![(
        u32::from_be_bytes(width.tag.to_bytes()),
        width.min_value,
    )]);
    let mut database = FontDatabase::default();
    let face = database
        .register_stored_face(descriptor, Arc::from(bytes.into_boxed_slice()), None)
        .expect("register variable descriptor face");

    assert_eq!(
        database.effective_variations(face, 700).unwrap(),
        VariationCoords(vec![
            (u32::from_be_bytes(width.tag.to_bytes()), width.min_value,),
            (
                u32::from_be_bytes(weight.tag.to_bytes()),
                700.0_f32.clamp(weight.min_value, weight.max_value),
            ),
        ])
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_instance_identity_quantizes_real_axis_to_f2dot14_bucket() {
    let source = Path::new(r"C:\Windows\Fonts\bahnschrift.ttf");
    let bytes = std::fs::read(source).expect("Windows variable-font fixture");
    let parsed = ttf_parser::Face::parse(&bytes, 0).expect("parse Bahnschrift");
    let width = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let negative_span = width.def_value - width.min_value;
    assert!(negative_span > 0.0);
    let first = width.def_value - negative_span * 0.25;
    let same_bucket = first - negative_span * (0.1 / 16_384.0);
    let tag = u32::from_be_bytes(width.tag.to_bytes());
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(source, Some("Bahnschrift Quantized Instance Test"), 0)
        .expect("register Windows variable font");

    assert_eq!(
        database
            .instance(face, &VariationCoords(vec![(tag, first)]))
            .unwrap(),
        database
            .instance(face, &VariationCoords(vec![(tag, same_bucket)]))
            .unwrap(),
        "coordinates in one OpenType normalized F2DOT14 bucket must share an instance"
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_asset_key_deduplicates_same_f2dot14_instance_bucket() {
    let source = Path::new(r"C:\Windows\Fonts\bahnschrift.ttf");
    let bytes = std::fs::read(source).expect("Windows variable-font fixture");
    let parsed = ttf_parser::Face::parse(&bytes, 0).expect("parse Bahnschrift");
    let width = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let span = width.def_value - width.min_value;
    let first = width.def_value - span * 0.25;
    let same_bucket = first - span * (0.1 / 16_384.0);
    let tag = u32::from_be_bytes(width.tag.to_bytes());
    let bytes: Arc<[u8]> = Arc::from(bytes.into_boxed_slice());
    let mut first_descriptor = FontFaceDescriptor::regular("Bahnschrift Asset Bucket Test");
    first_descriptor.variations = VariationCoords(vec![(tag, first)]);
    let mut second_descriptor = first_descriptor.clone();
    second_descriptor.variations = VariationCoords(vec![(tag, same_bucket)]);
    let logical_source = Path::new(r"C:\virtual\bahnschrift-variable.ttf");
    let mut database = FontDatabase::default();

    assert_eq!(
        database
            .register_asset_descriptor(first_descriptor, Arc::clone(&bytes), logical_source)
            .unwrap(),
        database
            .register_asset_descriptor(second_descriptor, bytes, logical_source)
            .unwrap(),
        "asset descriptors in one rendered variation bucket must share a base face"
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
fn text_font_database_maps_backend_face_ids_bidirectionally() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraMono-subset.ttf");
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(&source_path, Some("Fira Mono"), 0)
        .unwrap();

    let backend_face = database
        .backend_face_id(face)
        .expect("registered SFNT face must have a backend ID");
    assert_eq!(database.font_face_id(backend_face), Some(face));

    let mut font_system = FontSystem::new();
    database
        .load_face_into_font_system(face, &mut font_system)
        .unwrap();
    assert!(font_system.db().face(backend_face).is_some());
}

#[test]
fn text_font_database_decodes_woff2_once_for_native_and_sdf_consumers() {
    let original = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf"),
    )
    .unwrap();
    let encoded = encode(&original, BrotliQuality::default()).unwrap();
    let source_path = std::env::temp_dir().join(format!(
        "zircon-runtime-text-font-database-{}-decode.woff2",
        std::process::id()
    ));
    std::fs::write(&source_path, encoded).unwrap();

    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(&source_path, Some("Fira Woff2"), 0)
        .unwrap();
    let repeated = database
        .register_font_file(&source_path, Some("Fira Woff2"), 0)
        .unwrap();
    let native_bytes = database.face_bytes(face).unwrap();
    let sdf_bytes = database.face_bytes(face).unwrap();

    assert_eq!(repeated, face);
    assert!(!native_bytes.starts_with(b"wOF2"));
    assert!(ttf_parser::Face::parse(native_bytes.as_ref(), 0).is_ok());
    assert!(Arc::ptr_eq(&native_bytes, &sdf_bytes));

    let _ = std::fs::remove_file(source_path);
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
fn text_font_ttc_nonzero_face_materializes_for_real_sdf_raster() {
    let collection_path = write_ttc_fixture();
    let mut database = FontDatabase::default();
    let bold = database
        .register_font_file(&collection_path, Some("Fira TTC SDF"), 1)
        .unwrap();

    let standalone = database.standalone_face_bytes(bold).unwrap();
    let parsed = ttf_parser::Face::parse(standalone.as_ref(), 0).unwrap();
    assert_eq!(parsed.weight().to_number(), 700);
    let sdf_font = fontsdf::Font::from_bytes(standalone.as_ref()).unwrap();
    let (metrics, pixels) = sdf_font.rasterize_sdf('A', 32.0);

    assert!(metrics.width > 0);
    assert!(metrics.height > 0);
    assert!(pixels.iter().any(|value| *value != 0));

    let _ = std::fs::remove_file(collection_path);
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_materializes_discovered_system_face_bytes() {
    let (_, database) = super::super::shared_font_database_snapshot();
    let face = database
        .match_face(&FontQuery::single_family("Microsoft YaHei UI"))
        .expect("Windows CJK system font")
        .face;

    let original = database.face_bytes(face).expect("system face bytes");
    let standalone = database
        .standalone_face_bytes(face)
        .expect("standalone system face");

    assert!(!original.is_empty());
    assert!(ttf_parser::Face::parse(standalone.as_ref(), 0).is_ok());
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
        composite_font: Some(CompositeFontDescriptor {
            default_family: FontFamilyName::from("Primary Sans"),
            sub_fonts: vec![SubFontRange {
                family: FontFamilyName::from("Composite Han"),
                scripts: vec![FontScript::Han],
                ranges: vec![(0x3400, 0x9FFF)],
                cultures: vec![FontCultureTag::from("zh-Hans")],
            }],
        }),
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
    let composite_han = database
        .register_stored_face(
            FontFaceDescriptor::regular("Composite Han"),
            Arc::from([10_u8, 11, 12].as_slice()),
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
    assert!(!database
        .fallback_families()
        .iter()
        .any(|family| family.as_str() == "Composite Han"));
    assert_ne!(
        database
            .fallback_candidates_for_codepoint('界', &fallback_query, None, Some("zh-Hans-CN"),)
            .first()
            .copied(),
        Some(composite_han)
    );

    database.set_project_composite_font(asset.composite_font.clone());
    assert_eq!(
        database
            .fallback_candidates_for_codepoint('界', &fallback_query, None, Some("zh-Hans-CN"),)
            .first()
            .copied(),
        Some(composite_han)
    );
}

#[test]
fn text_font_database_composite_activation_is_explicit_and_replaceable() {
    let source_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraMono-subset.ttf");
    let mut database = FontDatabase::default();
    let primary_han = database
        .register_stored_face(
            FontFaceDescriptor::regular("Primary Han"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .unwrap();
    let secondary_han = database
        .register_stored_face(
            FontFaceDescriptor::regular("Secondary Han"),
            Arc::from([2_u8].as_slice()),
            None,
        )
        .unwrap();
    let primary = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Primary Han"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Primary Han"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x3400, 0x9FFF)],
            cultures: Vec::new(),
        }],
    };
    let secondary = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Secondary Han"),
        sub_fonts: vec![SubFontRange {
            family: FontFamilyName::from("Secondary Han"),
            scripts: vec![FontScript::Han],
            ranges: vec![(0x3400, 0x9FFF)],
            cultures: Vec::new(),
        }],
    };
    let secondary_asset = FontAsset {
        source: "FiraMono-subset.ttf".to_string(),
        family: Some("Secondary Asset Face".to_string()),
        render_mode: None,
        face_index: 0,
        family_members: Vec::new(),
        variable_instances: Vec::new(),
        fallback_families: Vec::new(),
        composite_font: Some(secondary.clone()),
        render_strategy: FontAssetRenderStrategy::default(),
        metadata: None,
    };
    let query = FontQuery::single_family("Missing Primary");

    database.set_project_composite_font(Some(primary));
    assert_eq!(
        database
            .fallback_candidates_for_codepoint('界', &query, None, None)
            .first()
            .copied(),
        Some(primary_han)
    );

    database
        .register_font_asset(&secondary_asset, &source_path)
        .unwrap();
    assert_eq!(
        database
            .fallback_candidates_for_codepoint('界', &query, None, None)
            .first()
            .copied(),
        Some(primary_han),
        "registering a secondary font asset must not replace project CompositeFont state"
    );

    database.set_project_composite_font(Some(secondary));
    assert_eq!(
        database
            .fallback_candidates_for_codepoint('界', &query, None, None)
            .first()
            .copied(),
        Some(secondary_han)
    );

    database.set_project_composite_font(None);
    let cleared = database.fallback_candidates_for_codepoint('界', &query, None, None);
    assert!(!cleared.contains(&primary_han));
    assert!(!cleared.contains(&secondary_han));
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
            cultures: Vec::new(),
        }],
    };

    let cjk_candidates =
        database.fallback_candidates_for_codepoint('界', &query, Some(&composite), None);
    assert_eq!(cjk_candidates.first().copied(), Some(cjk));
    assert!(cjk_candidates.contains(&latin));

    let latin_candidates =
        database.fallback_candidates_for_codepoint('A', &query, Some(&composite), None);
    assert_eq!(latin_candidates.first().copied(), Some(latin));
}

#[test]
fn text_composite_font_resolves_default_and_subfont_ranges() {
    let mut database = FontDatabase::default();
    let latin = database
        .register_stored_face(
            FontFaceDescriptor::regular("Fira Mono"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .unwrap();
    let simplified_han = database
        .register_stored_face(
            FontFaceDescriptor::regular("Noto Sans CJK SC"),
            Arc::from([2_u8].as_slice()),
            None,
        )
        .unwrap();
    let japanese = database
        .register_stored_face(
            FontFaceDescriptor::regular("Noto Sans CJK JP"),
            Arc::from([3_u8].as_slice()),
            None,
        )
        .unwrap();
    let query = FontQuery::single_family("Fira Mono");
    let composite = CompositeFontDescriptor {
        default_family: FontFamilyName::from("Fira Mono"),
        sub_fonts: vec![
            SubFontRange {
                family: FontFamilyName::from("Noto Sans CJK SC"),
                scripts: vec![FontScript::Han],
                ranges: vec![(0x3400, 0x9FFF)],
                cultures: vec![FontCultureTag::from("zh-Hans")],
            },
            SubFontRange {
                family: FontFamilyName::from("Noto Sans CJK JP"),
                scripts: vec![FontScript::Han],
                ranges: vec![(0x3400, 0x9FFF)],
                cultures: vec![FontCultureTag::from("ja")],
            },
        ],
    };

    assert_eq!(
        database
            .fallback_candidates_for_codepoint('界', &query, Some(&composite), Some("zh-Hans-CN"),)
            .first()
            .copied(),
        Some(simplified_han)
    );
    assert_eq!(
        database
            .fallback_candidates_for_codepoint('界', &query, Some(&composite), Some("ja-JP"))
            .first()
            .copied(),
        Some(japanese)
    );
    assert_eq!(
        database
            .fallback_candidates_for_codepoint('A', &query, Some(&composite), Some("zh-Hans"),)
            .first()
            .copied(),
        Some(latin)
    );
}

#[test]
fn text_font_runtime_default_composite_selects_checked_in_zh_hans_face() {
    let font_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let manifest_path = font_dir.join("default.font.toml");
    let asset = FontAsset::from_toml_str(
        &std::fs::read_to_string(manifest_path).expect("runtime default font manifest"),
    )
    .expect("parse runtime default font manifest");
    let source_path = font_dir.join(&asset.source);
    let mut database = FontDatabase::default();
    database
        .register_font_asset(&asset, &source_path)
        .expect("register checked-in default CompositeFont source");
    database.set_project_composite_font(asset.composite_font.clone());

    let selected = database
        .fallback_candidates_for_codepoint(
            '界',
            &FontQuery::single_family("Fira Mono"),
            None,
            Some("zh-Hans-CN"),
        )
        .first()
        .copied()
        .expect("checked-in zh-Hans CompositeFont candidate");

    assert_eq!(
        database
            .face_family_name(selected)
            .as_ref()
            .map(|family| family.as_str()),
        Some("Zircon Noto Sans CJK SC Proof")
    );
    assert_eq!(database.face_index(selected).unwrap(), 1);
    let bytes = database.face_bytes(selected).unwrap();
    let face = ttf_parser::Face::parse(bytes.as_ref(), 1).expect("checked-in CJK TTC face");
    assert!("中文排版引擎文本与布局"
        .chars()
        .all(|character| face.glyph_index(character).is_some()));
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
            cultures: Vec::new(),
        }],
    };

    let cjk_face =
        database.resolve_fallback_face_for_codepoint(primary, '界', &query, Some(&composite), None);
    let latin_face =
        database.resolve_fallback_face_for_codepoint(primary, 'A', &query, Some(&composite), None);

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

    let latin_candidates = database.fallback_candidates_for_codepoint('A', &query, None, None);
    let cjk_candidates = database.fallback_candidates_for_codepoint('界', &query, None, None);

    assert!(latin_candidates.contains(&latin));
    assert!(latin_candidates.contains(&unknown));
    assert!(!cjk_candidates.contains(&latin));
    assert!(cjk_candidates.contains(&unknown));
}
