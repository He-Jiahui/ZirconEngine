use super::*;

#[test]
fn text_font_database_system_font_policy_defaults_to_disabled() {
    let mut database = FontDatabase::default();

    assert_eq!(
        database.apply_system_font_policy(SystemFontPolicy::default()),
        0
    );
    assert!(database.faces.is_empty());
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_system_font_discovery_is_idempotent() {
    let mut database = FontDatabase::with_default_fallbacks();
    let system_query = FontQuery::single_family("Microsoft YaHei UI");
    assert!(database.match_face(&system_query).is_none());
    assert!(database.apply_system_font_policy(SystemFontPolicy::Discover) > 0);
    let registered_face_count = database.face_count();
    let backend_face_count = database.backend_database.faces().count();

    assert_eq!(
        database.apply_system_font_policy(SystemFontPolicy::Discover),
        0
    );
    assert_eq!(database.face_count(), registered_face_count);
    assert_eq!(
        database.backend_database.faces().count(),
        backend_face_count,
        "repeated discovery must not append duplicate backend faces"
    );
    assert!(
        database.match_face(&system_query).is_some(),
        "one batch-level cache detach must invalidate the pre-discovery miss"
    );
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
