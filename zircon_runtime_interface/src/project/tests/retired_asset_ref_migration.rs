use serde_json::json;

use crate::project::{
    migrate_retired_asset_references, migrate_retired_asset_references_with, AssetRef, RelPath,
    RetiredAssetRefMigrationError,
};

#[test]
fn exact_retired_reference_shape_uses_the_supplied_resolver() {
    let retired = json!({
        "component": {
            "uuid": "11111111-2222-4333-8444-555555555555",
            "url": "res://models/hero.glb#Mesh0"
        }
    });

    let migrated = migrate_retired_asset_references_with(retired, |reference| {
        assert_eq!(
            reference.locator().to_string(),
            "res://models/hero.glb#Mesh0"
        );
        AssetRef::try_new(
            reference.guid(),
            RelPath::parse("content/models/hero.glb").unwrap(),
            reference.locator().label().map(str::to_string),
        )
        .map_err(|error| error.to_string())
    })
    .unwrap();

    assert_eq!(
        migrated,
        json!({
            "component": {
                "guid": "11111111-2222-4333-8444-555555555555",
                "path_hint": "content/models/hero.glb",
                "sub": "Mesh0"
            }
        })
    );
}

#[test]
fn retired_builtin_reference_uses_the_distinct_builtin_contract() {
    let retired = json!({
        "shader": {
            "uuid": "11111111-2222-4333-8444-555555555555",
            "url": "builtin://shader/pbr.wgsl"
        }
    });

    let migrated = migrate_retired_asset_references(retired).unwrap();

    assert_eq!(
        migrated,
        json!({
            "shader": {
                "kind": "builtin",
                "locator": "builtin://shader/pbr.wgsl"
            }
        })
    );
}

#[test]
fn lookalike_objects_are_not_guessed_as_retired_references() {
    let lookalike = json!({
        "uuid": "11111111-2222-4333-8444-555555555555",
        "url": "res://models/hero.glb",
        "label": "ordinary domain object"
    });

    let migrated = migrate_retired_asset_references_with::<String>(lookalike.clone(), |_| {
        panic!("the resolver must not see non-exact shapes")
    })
    .unwrap();

    assert_eq!(migrated, lookalike);
}

#[test]
fn malformed_exact_shape_returns_a_typed_shape_error() {
    let error = migrate_retired_asset_references_with::<String>(
        json!({"uuid": 7, "url": "res://models/hero.glb"}),
        |_| panic!("malformed references must not reach the resolver"),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        RetiredAssetRefMigrationError::InvalidShape { .. }
    ));
}
