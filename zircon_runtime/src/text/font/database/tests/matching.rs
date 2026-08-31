use super::*;

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
fn text_font_database_default_match_precedence_and_cache_invalidation() {
    let mut database = FontDatabase::default();
    let runtime = database
        .register_stored_face(
            FontFaceDescriptor::regular("Runtime Default"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .expect("register runtime default face");
    let project = database
        .register_stored_face(
            FontFaceDescriptor::regular("Project Default"),
            Arc::from([2_u8].as_slice()),
            None,
        )
        .expect("register project default face");
    assert!(database.set_runtime_default_primary_face(runtime));
    assert!(database.set_runtime_default_ui_family("Runtime Default"));
    let unspecified = FontQuery::single_family("");

    assert_eq!(
        database
            .match_face(&unspecified)
            .map(|matched| matched.face),
        Some(runtime),
        "an unspecified family must use the engine-owned runtime default"
    );

    assert!(database.set_default_ui_family("Project Default"));
    assert_eq!(
        database
            .match_face(&unspecified)
            .map(|matched| matched.face),
        Some(project),
        "a project default must replace a cached runtime-default match"
    );

    assert!(database.clear_default_ui_family());
    assert_eq!(
        database
            .match_face(&unspecified)
            .map(|matched| matched.face),
        Some(runtime),
        "clearing the project default must restore the runtime face"
    );
    assert_eq!(
        database
            .match_face(&FontQuery::single_family("Project Default"))
            .map(|matched| matched.face),
        Some(project),
        "an explicit family must keep precedence over the runtime default"
    );
}

#[test]
fn text_font_database_private_alias_selects_packaged_face_without_shadowing_system_family() {
    let mut database = FontDatabase::default();
    let bytes: Arc<[u8]> = Arc::from(
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("assets/fonts/FiraSans-Regular.ttf"),
        )
        .expect("read a real font source for glyphon backend registration"),
    );
    let system = database
        .register_stored_face(
            FontFaceDescriptor::regular("Fira Mono"),
            Arc::clone(&bytes),
            None,
        )
        .expect("register system Fira Mono face");
    let packaged = database
        .register_stored_face(FontFaceDescriptor::regular("Fira Mono"), bytes, None)
        .expect("register packaged Fira Mono face");
    assert!(database.register_font_family_alias(
        packaged,
        FontFamilyName::from("Zircon Runtime Fallback Mono"),
    ));

    assert_eq!(
        database
            .match_face(&FontQuery::single_family("Fira Mono"))
            .map(|matched| matched.face),
        Some(system),
        "an explicit user family must keep its system face when the packaged face has the same name"
    );
    assert_eq!(
        database
            .match_face(&FontQuery::single_family("Zircon Runtime Fallback Mono"))
            .map(|matched| matched.face),
        Some(packaged),
        "retained fallback must address the exact packaged bytes through its private runtime alias"
    );
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
fn text_font_database_match_cache_isolated_after_cloned_generations_diverge() {
    let base = FontDatabase::default();
    let mut branch_a = base.clone();
    let mut branch_b = base;
    let face_a = branch_a
        .register_stored_face(
            FontFaceDescriptor::regular("Branch A"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .unwrap();
    let face_b = branch_b
        .register_stored_face(
            FontFaceDescriptor::regular("Branch B"),
            Arc::from([2_u8].as_slice()),
            None,
        )
        .unwrap();
    assert_eq!(
        face_a, face_b,
        "diverged clones deliberately reuse one numeric face ID"
    );
    let query = FontQuery::single_family("Branch A");

    assert_eq!(
        branch_a.match_face(&query).map(|matched| matched.face),
        Some(face_a)
    );
    assert!(
        branch_b.match_face(&query).is_none(),
        "a match cached by one clone generation must not leak into another"
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
