use super::*;

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
    let source_path = unique_font_fixture_path("font-database-decode", "woff2");
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
    let repeated_standalone = database.standalone_face_bytes(bold).unwrap();
    let parsed = ttf_parser::Face::parse(standalone.as_ref(), 0).unwrap();
    assert!(Arc::ptr_eq(&standalone, &repeated_standalone));
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
    let (_, database) = super::super::super::shared_font_database_snapshot();
    let face = database
        .match_face(&FontQuery::single_family("Microsoft YaHei UI"))
        .expect("Windows CJK system font")
        .face;

    let original = database.face_bytes(face).expect("system face bytes");
    let repeated_original = database.face_bytes(face).expect("cached system face bytes");
    let standalone = database
        .standalone_face_bytes(face)
        .expect("standalone system face");
    let repeated_standalone = database
        .standalone_face_bytes(face)
        .expect("cached standalone system face");

    assert!(!original.is_empty());
    assert!(Arc::ptr_eq(&original, &repeated_original));
    assert!(Arc::ptr_eq(&standalone, &repeated_standalone));
    assert!(ttf_parser::Face::parse(standalone.as_ref(), 0).is_ok());
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_builds_discovered_system_face_metadata_once_on_first_use() {
    let mut database = FontDatabase::default();
    assert!(database.apply_system_font_policy(SystemFontPolicy::Discover) > 0);
    let face = database
        .match_face(&FontQuery::single_family("Microsoft YaHei UI"))
        .expect("Windows CJK system font")
        .face;
    assert_eq!(database.face_metadata_build_count(), 0);

    let source_identity = database
        .face_source_identity(face)
        .expect("system face source identity");
    assert_ne!(source_identity, [0; 16]);
    assert!(database
        .face_metrics(face)
        .expect("system face metrics")
        .is_some());
    assert_eq!(database.face_metadata_build_count(), 1);

    assert_eq!(
        database.face_source_identity(face).unwrap(),
        source_identity
    );
    assert!(database.face_metrics(face).unwrap().is_some());
    assert_eq!(database.face_metadata_build_count(), 1);
}
