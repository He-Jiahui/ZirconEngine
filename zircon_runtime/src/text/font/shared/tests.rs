use super::*;
use crate::text::{CompositeFontDescriptor, FontFamilyName};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn identical_shared_font_database_publish_preserves_generation() {
    let shared = SharedFontDatabase::from_database(FontDatabase::with_default_fallbacks());
    let (generation, _) = shared.snapshot();

    let (first, _, ()) = shared.mutate(|_| ());
    let (second, _, ()) = shared.mutate(|_| ());

    assert_eq!(first, generation);
    assert_eq!(second, generation);
}

#[test]
fn changed_shared_font_database_publish_advances_generation_once() {
    let shared = SharedFontDatabase::from_database(FontDatabase::with_default_fallbacks());
    let (generation, _) = shared.snapshot();

    let (changed, _, did_change) = shared.mutate(|database| {
        let changed = database.set_project_composite_font(Some(CompositeFontDescriptor {
            default_family: FontFamilyName::from("Project UI"),
            sub_fonts: Vec::new(),
        }));
        changed
    });
    let (unchanged, _, did_change_again) = shared.mutate(|database| {
        let changed = database.set_project_composite_font(Some(CompositeFontDescriptor {
            default_family: FontFamilyName::from("Project UI"),
            sub_fonts: Vec::new(),
        }));
        changed
    });

    assert!(changed > generation);
    assert_eq!(unchanged, changed);
    assert!(did_change);
    assert!(!did_change_again);
}

#[test]
fn changed_default_ui_family_advances_shared_font_generation_once() {
    let shared = SharedFontDatabase::from_database(FontDatabase::with_default_fallbacks());
    let (generation, _) = shared.snapshot();

    let (changed, _, did_change) = shared.mutate(|database| {
        let changed = database.set_default_ui_family("Project UI");
        changed
    });
    let (unchanged, _, did_change_again) = shared.mutate(|database| {
        let changed = database.set_default_ui_family("Project UI");
        changed
    });

    assert!(changed > generation);
    assert_eq!(unchanged, changed);
    assert!(did_change);
    assert!(!did_change_again);
}

#[test]
fn shared_font_asset_mutations_preserve_independent_owners() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    let destination = std::env::temp_dir().join(format!(
        "zircon-shared-font-revision-{}-{nonce}.ttf",
        std::process::id()
    ));
    std::fs::copy(source, &destination).expect("copy shared font fixture");
    let first_owner = "res://fonts/shared-first.font.toml";
    let second_owner = "res://fonts/shared-second.font.toml";
    let shared = SharedFontDatabase::from_database(FontDatabase::default());
    let (initial_generation, _) = shared.snapshot();
    let (first_generation, _, first) = shared.mutate(|database| {
        let report = database
            .replace_font_source(first_owner, &destination, Some("Shared Reloadable"), 0)
            .expect("register first shared owner");
        report
    });
    let (second_generation, _, attached) = shared.mutate(|database| {
        let report = database
            .replace_font_source(second_owner, &destination, Some("Shared Reloadable"), 0)
            .expect("attach independent shared owner");
        report
    });

    assert!(first.database_changed);
    assert!(first.asset_mapping_changed);
    assert!(first_generation > initial_generation);
    assert!(!attached.database_changed);
    assert!(attached.asset_mapping_changed);
    assert_eq!(second_generation, first_generation);

    let (removal_generation, remaining, first_owner_removed) = shared.mutate(|database| {
        let report = database.remove_font_asset(first_owner);
        report
    });

    assert!(!first_owner_removed.database_changed);
    assert!(first_owner_removed.asset_mapping_changed);
    assert_eq!(removal_generation, second_generation);
    assert_eq!(remaining.face_count(), 1);

    OpenOptions::new()
        .append(true)
        .open(&destination)
        .expect("open shared fixture for revision")
        .write_all(&[0])
        .expect("append shared revision byte");
    let (replacement_generation, _, replacement) = shared.mutate(|database| {
        let report = database
            .replace_font_source(second_owner, &destination, Some("Shared Reloadable"), 0)
            .expect("replace remaining shared owner revision");
        report
    });
    let (final_generation, final_database, removed) = shared.mutate(|database| {
        let report = database.remove_font_asset(second_owner);
        report
    });

    assert!(replacement.database_changed);
    assert!(replacement.asset_mapping_changed);
    assert!(replacement_generation > removal_generation);
    assert!(removed.database_changed);
    assert!(removed.asset_mapping_changed);
    assert!(final_generation > replacement_generation);
    assert_eq!(final_database.face_count(), 0);

    let _ = std::fs::remove_file(destination);
}
