use super::*;

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
