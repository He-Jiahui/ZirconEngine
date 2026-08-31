use super::*;
#[cfg(feature = "profiling")]
use crate::core::diagnostics::profiling::{
    ProfileCaptureConfig, reset_capture, snapshot, start_capture, test_capture_lock,
};
use crate::text::font::{
    force_publish_shared_font_database, shared_font_collection_snapshot,
    shared_font_database_snapshot, shared_font_database_test_serial_guard,
};

struct SharedFontDatabaseRestore(FontDatabase);

impl Drop for SharedFontDatabaseRestore {
    fn drop(&mut self) {
        force_publish_shared_font_database(&self.0);
    }
}

#[test]
fn locale_font_system_cache_uses_the_supplied_generation_snapshot() {
    let _font_database_guard = shared_font_database_test_serial_guard();
    let retired = shared_font_collection_snapshot();
    let retired_family = retired
        .database()
        .default_ui_family_for_test()
        .map(str::to_owned);
    let (_, original_database) = shared_font_database_snapshot();
    let _restore = SharedFontDatabaseRestore(original_database.clone());
    let mut replacement = original_database.clone();
    assert!(replacement.set_default_ui_family("Generation Snapshot Test UI"));
    let replacement_generation = force_publish_shared_font_database(&replacement);
    assert!(replacement_generation > retired.generation());

    let mut cache = LocaleFontSystemCache::new(&retired);
    cache.with_font_system(&retired, Some("en-US"), |_, database| {
        assert_eq!(
            database.default_ui_family_for_test().map(str::to_owned),
            retired_family,
            "cosmic shaping must read the database retained by the supplied snapshot"
        );
    });
}

#[cfg(feature = "profiling")]
#[test]
fn locale_font_system_cache_profiles_insert_and_generation_refresh() {
    let _capture_guard = test_capture_lock();
    let _font_database_guard = shared_font_database_test_serial_guard();
    let initial_snapshot = shared_font_collection_snapshot();
    let mut cache = LocaleFontSystemCache::new(&initial_snapshot);
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "text01-locale-font-system-cache".to_owned();
    config.max_spans = 4;
    config.max_counters = 8;
    assert!(start_capture(config).active);

    cache.with_font_system(&initial_snapshot, Some("zh-Hans"), |_, _| ());
    let (_, database) = shared_font_database_snapshot();
    force_publish_shared_font_database(&database);
    let reloaded_snapshot = shared_font_collection_snapshot();
    cache.with_font_system(&reloaded_snapshot, Some("ja-JP"), |_, _| ());

    let profile = snapshot();
    assert!(!reset_capture().active);
    for name in ["locale_insert", "generation_refresh"] {
        assert!(
            profile.spans.iter().any(|span| {
                span.stream == "runtime"
                    && span.category == "text.font_system_cache"
                    && span.name == name
            }),
            "locale font-system cache must expose {name} work to profiling"
        );
    }
    for name in [
        "text.font_system_cache.locale_insert_entry_count",
        "text.font_system_cache.generation_refresh_entry_count",
        "text.font_system_cache.generation_refresh_face_count",
    ] {
        assert!(
            profile
                .counters
                .iter()
                .any(|counter| counter.stream == "runtime" && counter.name == name),
            "locale font-system cache must emit {name}"
        );
    }
}
