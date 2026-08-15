use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use glyphon::{Attrs, Buffer, Metrics, Shaping};

use super::*;
use crate::asset::FontAsset;
use crate::text::font::test_font_fixtures::unique_font_fixture_path;
use crate::text::{CompositeFontDescriptor, FontFamilyName, FontQuery};

#[test]
fn runtime_shared_font_database_bootstraps_default_manifest_and_private_alias() {
    let mut database = runtime_default_font_database();
    let face = database
        .font_asset_primary_face(PACKAGED_DEFAULT_FONT_OWNER)
        .expect("checked-in packaged default font face");
    let font_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");
    let asset = FontAsset::from_toml_str(
        &std::fs::read_to_string(font_dir.join("default.font.toml"))
            .expect("runtime default font manifest"),
    )
    .expect("parse runtime default font manifest");
    let before_attach = database.face_count();
    assert_eq!(
        before_attach, 2,
        "the Runtime bootstrap must preload exactly the manifest's Fira Mono and CJK faces"
    );
    let attached = database
        .replace_font_asset(
            "res://fonts/default.font.toml",
            &asset,
            font_dir.join(&asset.source),
        )
        .expect("attach default UI asset owner to packaged faces");

    assert_eq!(
        database
            .face_family_name(face)
            .as_ref()
            .map(FontFamilyName::as_str),
        Some(PACKAGED_DEFAULT_FONT_FAMILY)
    );
    assert!(attached.asset_mapping_changed);
    assert!(
        !attached.database_changed,
        "the UI owner must attach to the bootstrap manifest faces instead of registering a duplicate TTC"
    );
    assert_eq!(attached.faces.len(), 2);
    assert_eq!(attached.faces[0], face);
    assert_eq!(
        database
            .face_family_name(attached.faces[0])
            .as_ref()
            .map(FontFamilyName::as_str),
        Some(PACKAGED_DEFAULT_FONT_FAMILY)
    );
    assert_eq!(
        database
            .face_family_name(attached.faces[1])
            .as_ref()
            .map(FontFamilyName::as_str),
        Some("Zircon Noto Sans CJK SC Proof")
    );
    assert_eq!(
        database.font_asset_primary_face("res://fonts/default.font.toml"),
        Some(face)
    );
    assert_eq!(database.face_count(), before_attach);

    let removed = database.remove_font_asset("res://fonts/default.font.toml");

    assert!(removed.asset_mapping_changed);
    assert!(
        !removed.database_changed,
        "removing the UI owner must preserve the permanent bootstrap faces"
    );
    assert!(removed.retired_faces.is_empty());
    assert_eq!(database.face_count(), before_attach);
    assert_eq!(
        database.font_asset_primary_face(PACKAGED_DEFAULT_FONT_OWNER),
        Some(face)
    );
    assert_eq!(
        database
            .match_face(&FontQuery::single_family(PACKAGED_RUNTIME_FALLBACK_FAMILY))
            .map(|matched| matched.face),
        Some(face),
        "the retained fallback identity must resolve the packaged TTC face, not a same-named system face"
    );
    let backend = database.backend_database_snapshot();
    let families = [glyphon::fontdb::Family::Name(
        PACKAGED_RUNTIME_FALLBACK_FAMILY,
    )];
    let backend_face = backend
        .query(&glyphon::fontdb::Query {
            families: &families,
            ..glyphon::fontdb::Query::default()
        })
        .expect("the runtime fallback identity must be queryable by glyphon");
    assert_eq!(database.font_face_id(backend_face), Some(face));

    let mut font_system = glyphon::FontSystem::new_with_locale_and_db(
        "en-us".to_string(),
        database.backend_database_snapshot(),
    );
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
    let attrs = Attrs::new().family(glyphon::Family::Name(PACKAGED_RUNTIME_FALLBACK_FAMILY));
    buffer.set_text(
        &mut font_system,
        "Fallback",
        &attrs,
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let shaped_backend_face = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .next()
        .expect("ASCII fallback text must shape a glyph")
        .font_id;
    assert_eq!(database.font_face_id(shaped_backend_face), Some(face));
}

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
    let destination = unique_font_fixture_path("shared-font-revision", "ttf");
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
