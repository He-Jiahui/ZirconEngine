use super::*;
use crate::text::{CompositeFontDescriptor, FontFamilyName};

#[test]
fn identical_shared_font_database_publish_preserves_generation() {
    let shared = SharedFontDatabase::from_database(FontDatabase::with_default_fallbacks());
    let (generation, database) = shared.snapshot();

    let first = shared.publish(&database);
    let second = shared.publish(&database);

    assert_eq!(first, generation);
    assert_eq!(second, generation);
}

#[test]
fn changed_shared_font_database_publish_advances_generation_once() {
    let shared = SharedFontDatabase::from_database(FontDatabase::with_default_fallbacks());
    let (generation, mut database) = shared.snapshot();
    database.set_project_composite_font(Some(CompositeFontDescriptor {
        default_family: FontFamilyName::from("Project UI"),
        sub_fonts: Vec::new(),
    }));

    let changed = shared.publish(&database);
    let unchanged = shared.publish(&database);

    assert!(changed > generation);
    assert_eq!(unchanged, changed);
}

#[test]
fn changed_default_ui_family_advances_shared_font_generation_once() {
    let shared = SharedFontDatabase::from_database(FontDatabase::with_default_fallbacks());
    let (generation, mut database) = shared.snapshot();
    database.set_default_ui_family("Project UI");

    let changed = shared.publish(&database);
    let unchanged = shared.publish(&database);

    assert!(changed > generation);
    assert_eq!(unchanged, changed);
}
