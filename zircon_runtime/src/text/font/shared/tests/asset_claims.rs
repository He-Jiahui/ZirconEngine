use std::path::Path;
use std::sync::Arc;

use super::super::*;
use crate::asset::ProjectAssetManager;
use crate::text::font::DEFAULT_UI_FONT_ASSET;
use crate::text::font::prepare_runtime_font_asset_admission;
use crate::text::font::test_font_fixtures::unique_font_fixture_path;
use crate::text::{CompositeFontDescriptor, FontFamilyName};

#[test]
fn runtime_font_asset_owner_survives_until_the_last_claim_scope_releases_it() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let destination = unique_font_fixture_path("runtime-font-asset-claims", "ttf");
    std::fs::copy(source, &destination).expect("copy claimed font fixture");
    let owner = "res://fonts/shared-claim.font.toml";
    let collection = FontCollectionService::from_database(FontDatabase::default());
    let mut first = collection.runtime_font_asset_claim_scope();
    let mut second = collection.runtime_font_asset_claim_scope();

    first.replace_claims([owner]);
    second.replace_claims([owner]);
    let (admitted_generation, _, report) = collection.mutate(|database| {
        database
            .replace_font_source(owner, &destination, Some("Claimed UI"), 0)
            .expect("register claimed font owner")
    });
    assert!(report.asset_mapping_changed);

    drop(first);
    let after_first_release = collection.collection_snapshot();
    assert_eq!(after_first_release.generation(), admitted_generation);
    assert!(after_first_release.database().has_font_asset_owner(owner));

    drop(second);
    let after_last_release = collection.collection_snapshot();
    assert_eq!(after_last_release.generation(), admitted_generation + 1);
    assert!(!after_last_release.database().has_font_asset_owner(owner));
    assert_eq!(after_last_release.database().face_count(), 0);

    let _ = std::fs::remove_file(destination);
}

#[test]
fn dropping_one_claim_scope_retires_all_unshared_owners_in_one_publication() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let destination = unique_font_fixture_path("runtime-font-asset-batch-release", "ttf");
    std::fs::copy(source, &destination).expect("copy batch-release font fixture");
    let custom_owner = "res://fonts/custom-project.font.toml";
    let collection = FontCollectionService::new();
    let runtime_face_count = collection.collection_snapshot().database().face_count();
    let mut scope = collection.runtime_font_asset_claim_scope();

    scope.replace_claims([DEFAULT_UI_FONT_ASSET, custom_owner]);
    let (admitted_generation, _, ()) = collection.mutate(|database| {
        database
            .replace_font_source(
                DEFAULT_UI_FONT_ASSET,
                &destination,
                Some("Project Default UI"),
                0,
            )
            .expect("register project default font owner");
        database
            .replace_font_source(custom_owner, &destination, Some("Project Custom UI"), 0)
            .expect("register custom project font owner");
        database.set_project_composite_font(Some(CompositeFontDescriptor {
            default_family: FontFamilyName::from("Project Default UI"),
            sub_fonts: Vec::new(),
        }));
        database.set_default_ui_family("Project Default UI");
    });

    drop(scope);
    let released = collection.collection_snapshot();
    assert_eq!(released.generation(), admitted_generation + 1);
    assert!(
        !released
            .database()
            .has_font_asset_owner(DEFAULT_UI_FONT_ASSET)
    );
    assert!(!released.database().has_font_asset_owner(custom_owner));
    assert_eq!(released.database().face_count(), runtime_face_count);
    assert_eq!(
        released.database().default_ui_family_for_test(),
        Some(PACKAGED_DEFAULT_FONT_FAMILY)
    );
    assert_eq!(released.database().project_composite_font_for_test(), None);

    let _ = std::fs::remove_file(destination);
}

#[test]
fn replacing_claimed_assets_retires_and_admits_in_one_publication() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let first_path = unique_font_fixture_path("runtime-font-asset-transition-first", "ttf");
    let second_path = unique_font_fixture_path("runtime-font-asset-transition-second", "ttf");
    std::fs::copy(&source, &first_path).expect("copy first transition font fixture");
    std::fs::copy(source, &second_path).expect("copy second transition font fixture");
    let first_owner = Arc::<str>::from(first_path.to_string_lossy().as_ref());
    let second_owner = Arc::<str>::from(second_path.to_string_lossy().as_ref());
    let asset_manager = ProjectAssetManager::default();
    let collection = FontCollectionService::from_database(FontDatabase::default());
    let mut scope = collection.runtime_font_asset_claim_scope();

    let first = prepare_runtime_font_asset_admission(&asset_manager, Arc::clone(&first_owner));
    let first_transition = scope
        .replace_shared_claims_with_admissions(std::slice::from_ref(&first_owner), vec![first]);
    assert!(first_transition.admissions[0].result.is_ok());
    let first_generation = collection.collection_snapshot().generation();

    let second = prepare_runtime_font_asset_admission(&asset_manager, Arc::clone(&second_owner));
    let second_transition = scope
        .replace_shared_claims_with_admissions(std::slice::from_ref(&second_owner), vec![second]);
    let published = collection.collection_snapshot();

    assert_eq!(second_transition.claims.unclaimed_asset_count, 1);
    assert!(second_transition.admissions[0].result.is_ok());
    assert_eq!(published.generation(), first_generation + 1);
    assert!(!published.database().has_font_asset_owner(&first_owner));
    assert!(published.database().has_font_asset_owner(&second_owner));

    let _ = std::fs::remove_file(first_path);
    let _ = std::fs::remove_file(second_path);
}
