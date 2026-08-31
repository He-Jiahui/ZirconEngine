use serde_json::json;

use crate::project::{
    migrate_retired_asset_references, migrate_retired_asset_references_with,
    migrate_retired_asset_references_with_budget, AssetRef, RelPath,
    RetiredAssetRefMigrationBudget, RetiredAssetRefMigrationError,
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

#[test]
fn programmatic_value_depth_is_rejected_before_the_resolver_runs() {
    let mut value = json!(null);
    for _ in 0..=128 {
        value = json!([value]);
    }

    let error = migrate_retired_asset_references_with::<String>(value, |_| {
        panic!("depth admission must run before reference resolution")
    })
    .unwrap_err();

    assert!(matches!(
        error,
        RetiredAssetRefMigrationError::ResourceLimitExceeded {
            resource: "retired asset migration depth",
            max: 128,
            found: 129,
        }
    ));
}

#[test]
fn caller_budget_rejects_nodes_before_the_resolver_runs() {
    let error = migrate_retired_asset_references_with_budget::<String>(
        json!([null, null]),
        RetiredAssetRefMigrationBudget::new(2, 128, 8),
        |_| panic!("node admission must run before reference resolution"),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        RetiredAssetRefMigrationError::ResourceLimitExceeded {
            resource: "retired asset migration nodes",
            max: 2,
            found: 3,
        }
    ));
}

#[test]
fn caller_budget_rejects_reference_count_before_the_resolver_runs() {
    let retired = || {
        json!({
            "uuid": "11111111-2222-4333-8444-555555555555",
            "url": "res://models/hero.glb"
        })
    };
    let error = migrate_retired_asset_references_with_budget::<String>(
        json!([retired(), retired()]),
        RetiredAssetRefMigrationBudget::new(16, 128, 1),
        |_| panic!("reference admission must complete before resolution"),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        RetiredAssetRefMigrationError::ResourceLimitExceeded {
            resource: "retired asset migration references",
            max: 1,
            found: 2,
        }
    ));
}
