use super::*;

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

    let registered = database
        .replace_font_asset("res://fonts/primary.font.toml", &asset, &source_path)
        .unwrap()
        .faces;
    let registered_again = database
        .replace_font_asset("res://fonts/primary.font.toml", &asset, &source_path)
        .unwrap()
        .faces;
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
        .replace_font_asset(
            "res://fonts/secondary.font.toml",
            &secondary_asset,
            &source_path,
        )
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
        .replace_font_asset("res://fonts/default.font.toml", &asset, &source_path)
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
