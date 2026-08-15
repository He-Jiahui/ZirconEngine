use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use super::*;
use crate::text::FontQuery;

#[test]
fn text_font_database_same_path_revision_replaces_face_and_removes_stale_indexes() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let destination = revision_fixture_path();
    std::fs::copy(source, &destination).expect("copy reloadable font fixture");
    let mut database = FontDatabase::default();
    let asset_ref = "res://fonts/reloadable.font.toml";

    let first = database
        .replace_font_source(asset_ref, &destination, Some("Reloadable Sans"), 0)
        .expect("register first font revision");
    let first_face = first.faces[0];
    let first_identity = database.face_source_identity(first_face).unwrap();
    let first_backend = database
        .backend_face_id(first_face)
        .expect("first revision backend face");
    let first_bytes = database
        .face_bytes(first_face)
        .expect("first revision source bytes");
    let first_bytes_weak = Arc::downgrade(&first_bytes);
    drop(first_bytes);

    OpenOptions::new()
        .append(true)
        .open(&destination)
        .expect("open font fixture for revision")
        .write_all(&[0])
        .expect("append semantically harmless revision byte");

    let second = database
        .replace_font_source(asset_ref, &destination, Some("Reloadable Sans"), 0)
        .expect("replace font revision");
    let second_face = second.faces[0];

    assert!(first.database_changed);
    assert!(first.asset_mapping_changed);
    assert!(second.database_changed);
    assert!(second.asset_mapping_changed);
    assert_eq!(second.retired_faces, vec![first_face]);
    assert_ne!(second_face, first_face);
    assert_ne!(
        database.face_source_identity(second_face).unwrap(),
        first_identity
    );
    assert_eq!(database.face_count(), 1);
    assert!(matches!(
        database.face_bytes(first_face),
        Err(FontDatabaseError::UnknownFace(face)) if face == first_face
    ));
    assert_eq!(database.backend_face_id(first_face), None);
    assert_eq!(database.font_face_id(first_backend), None);
    assert!(
        first_bytes_weak.upgrade().is_none(),
        "retired font revisions must release their source bytes"
    );
    assert_eq!(
        database
            .match_face(&FontQuery::single_family("Reloadable Sans"))
            .expect("replacement face should remain matchable")
            .face,
        second_face
    );

    let removed = database.remove_font_asset(asset_ref);

    assert!(removed.database_changed);
    assert!(removed.asset_mapping_changed);
    assert_eq!(removed.retired_faces, vec![second_face]);
    assert_eq!(database.face_count(), 0);
    assert!(database
        .match_face(&FontQuery::single_family("Reloadable Sans"))
        .is_none());

    let repeated = database.remove_font_asset(asset_ref);
    assert!(!repeated.database_changed);
    assert!(!repeated.asset_mapping_changed);
    assert!(repeated.faces.is_empty());
    assert!(repeated.retired_faces.is_empty());

    let _ = std::fs::remove_file(destination);
}

#[test]
fn text_font_database_shared_face_tracks_owner_mapping_without_render_input_change() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let mut database = FontDatabase::default();
    let first_owner = "res://fonts/shared-first.font.toml";
    let second_owner = "res://fonts/shared-second.font.toml";

    let first = database
        .replace_font_source(first_owner, &source, Some("Shared UI"), 0)
        .expect("register first shared owner");
    let second = database
        .replace_font_source(second_owner, &source, Some("Shared UI"), 0)
        .expect("register second shared owner");

    assert!(first.database_changed);
    assert!(first.asset_mapping_changed);
    assert!(!second.database_changed);
    assert!(second.asset_mapping_changed);
    assert_eq!(database.face_count(), 1);
    assert_eq!(
        database.font_asset_primary_face(first_owner),
        Some(first.faces[0]),
        "the original owner must resolve its registered active face without a manifest reload"
    );
    assert_eq!(
        database.font_asset_primary_face(second_owner),
        Some(first.faces[0]),
        "shared source owners must resolve the same active face"
    );

    let first_removed = database.remove_font_asset(first_owner);
    assert!(!first_removed.database_changed);
    assert!(first_removed.asset_mapping_changed);
    assert!(first_removed.retired_faces.is_empty());
    assert_eq!(database.face_count(), 1);
    assert_eq!(database.font_asset_primary_face(first_owner), None);
    assert_eq!(
        database.font_asset_primary_face(second_owner),
        Some(first.faces[0])
    );

    let repeated = database.remove_font_asset(first_owner);
    assert!(!repeated.database_changed);
    assert!(!repeated.asset_mapping_changed);

    let second_removed = database.remove_font_asset(second_owner);
    assert!(second_removed.database_changed);
    assert!(second_removed.asset_mapping_changed);
    assert_eq!(database.face_count(), 0);
    assert_eq!(database.font_asset_primary_face(second_owner), None);
}

#[test]
fn text_font_database_removing_final_owner_removes_private_family_alias() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let owner = "res://fonts/aliased.font.toml";
    let alias = FontFamilyName::from("Zircon Test Private Alias");
    let mut database = FontDatabase::default();
    let registered = database
        .replace_font_source(owner, &source, Some("Aliased Source"), 0)
        .expect("register aliased source");
    let face = registered.faces[0];
    assert_eq!(database.backend_database_snapshot().faces().count(), 1);

    assert!(database.register_font_family_alias(face, alias.clone()));
    assert_eq!(database.backend_database_snapshot().faces().count(), 2);
    assert_eq!(
        database
            .match_face(&FontQuery::single_family(alias.as_str()))
            .map(|matched| matched.face),
        Some(face)
    );
    let backend = database.backend_database_snapshot();
    let families = [fontdb::Family::Name(alias.as_str())];
    let backend_alias = backend
        .query(&fontdb::Query {
            families: &families,
            ..fontdb::Query::default()
        })
        .expect("private aliases must be visible to the glyphon font database");
    assert_eq!(database.font_face_id(backend_alias), Some(face));

    database.remove_font_asset(owner);

    assert!(database
        .match_face(&FontQuery::single_family(alias.as_str()))
        .is_none());
    assert!(
        database
            .backend_database_snapshot()
            .query(&fontdb::Query {
                families: &families,
                ..fontdb::Query::default()
            })
            .is_none(),
        "retiring the final logical owner must remove the glyphon alias entry too"
    );
    assert_eq!(database.backend_database_snapshot().faces().count(), 0);
}

#[test]
fn text_font_database_removing_asset_fallback_invalidates_shared_face_render_inputs() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let shared_owner = "res://fonts/shared-owner.font.toml";
    let fallback_owner = "res://fonts/fallback-owner.font.toml";
    let mut database = FontDatabase::default();

    database
        .replace_font_source(shared_owner, &source, Some("Shared UI"), 0)
        .expect("register shared face owner");
    let asset = FontAsset {
        source: "FiraSans-Regular.ttf".to_string(),
        family: Some("Shared UI".to_string()),
        render_mode: None,
        face_index: 0,
        family_members: Vec::new(),
        variable_instances: Vec::new(),
        fallback_families: vec!["Asset Fallback".to_string()],
        composite_font: None,
        render_strategy: FontAssetRenderStrategy::default(),
        metadata: None,
    };
    let registered = database
        .replace_font_asset(fallback_owner, &asset, &source)
        .expect("register fallback owner");

    assert!(registered.database_changed);
    assert_eq!(database.face_count(), 1);
    assert!(database
        .fallback_families()
        .iter()
        .any(|family| family.as_str() == "Asset Fallback"));

    let removed = database.remove_font_asset(fallback_owner);

    assert!(removed.database_changed);
    assert!(removed.asset_mapping_changed);
    assert!(removed.retired_faces.is_empty());
    assert_eq!(database.face_count(), 1);
    assert!(!database
        .fallback_families()
        .iter()
        .any(|family| family.as_str() == "Asset Fallback"));
}

#[test]
fn text_font_asset_ttc_registration_and_removal_leave_no_unowned_backend_faces() {
    let source_path = write_ttc_fixture();
    let source_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("TTC fixture name should be UTF-8")
        .to_string();
    let asset = FontAsset {
        source: source_name,
        family: Some("Reloadable TTC".to_string()),
        render_mode: None,
        face_index: 0,
        family_members: vec![
            FontAssetFamilyMember {
                family: "Reloadable TTC Regular".to_string(),
                face_index: 0,
                weight: Some(400),
                width_class: None,
                style: None,
                variations: Vec::new(),
            },
            FontAssetFamilyMember {
                family: "Reloadable TTC Bold".to_string(),
                face_index: 1,
                weight: Some(700),
                width_class: None,
                style: None,
                variations: Vec::new(),
            },
        ],
        variable_instances: Vec::new(),
        fallback_families: Vec::new(),
        composite_font: None,
        render_strategy: FontAssetRenderStrategy::default(),
        metadata: None,
    };
    let mut database = FontDatabase::default();
    let asset_ref = "res://fonts/reloadable-collection.font.toml";

    let registered = database
        .replace_font_asset(asset_ref, &asset, &source_path)
        .expect("register TTC asset faces");

    assert_eq!(registered.faces.len(), 2);
    assert_eq!(database.face_count(), 2);
    assert_eq!(database.backend_database_snapshot().faces().count(), 2);

    let removed = database.remove_font_asset(asset_ref);

    assert_eq!(removed.retired_faces.len(), 2);
    assert_eq!(database.face_count(), 0);
    assert_eq!(database.backend_database_snapshot().faces().count(), 0);

    let _ = std::fs::remove_file(source_path);
}

fn revision_fixture_path() -> std::path::PathBuf {
    unique_font_fixture_path("font-asset-revision", "ttf")
}
