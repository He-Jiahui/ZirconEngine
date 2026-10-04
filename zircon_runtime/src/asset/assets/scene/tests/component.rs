use super::SceneComponentAssetRecord;
use crate::asset::{AssetReference, AssetUri};

#[test]
fn generic_component_row_keeps_stable_identity_and_maps_nested_references() {
    let reference =
        AssetReference::from_locator(AssetUri::parse("builtin://texture/checkerboard").unwrap());
    let row = SceneComponentAssetRecord::from_typed(
        "tests.render.Sprite",
        "tests.render.Sprite.v1",
        1,
        "tests.render",
        &serde_json::json!({
            "image": reference,
            "nested": [{"material": reference}],
            "size": [16.0, 8.0]
        }),
    )
    .unwrap();

    assert_eq!(row.type_id, "tests.render.Sprite");
    assert_eq!(row.schema_id, "tests.render.Sprite.v1");
    assert_eq!(row.schema_version, 1);
    assert_eq!(row.provider_id, "tests.render");
    assert_eq!(row.references, vec![reference.clone(), reference.clone()]);
    assert_eq!(
        row.decode_typed::<serde_json::Value>().unwrap()["image"]["url"],
        "builtin://texture/checkerboard"
    );
}

#[test]
fn generic_component_row_rejects_reference_slot_corruption_and_marker_collision() {
    let malformed = SceneComponentAssetRecord {
        type_id: "tests.Component".into(),
        schema_id: "tests.Component.v1".into(),
        schema_version: 1,
        provider_id: "tests".into(),
        payload: serde_json::json!({
            "$zircon_scene_asset_reference": 4
        }),
        references: Vec::new(),
    };
    assert!(malformed.decode_typed::<serde_json::Value>().is_err());

    assert!(SceneComponentAssetRecord::from_typed(
        "tests.Component",
        "tests.Component.v1",
        1,
        "tests",
        &serde_json::json!({"$zircon_scene_asset_reference": 0}),
    )
    .is_err());
    assert!(SceneComponentAssetRecord::from_typed(
        "tests.Component",
        "tests.Component.v1",
        1,
        "tests",
        &serde_json::json!({"$zircon_scene_asset_reference": 0, "extra": true}),
    )
    .is_err());
    let malformed_marker = SceneComponentAssetRecord {
        type_id: "tests.Component".into(),
        schema_id: "tests.Component.v1".into(),
        schema_version: 1,
        provider_id: "tests".into(),
        payload: serde_json::json!({
            "$zircon_scene_asset_reference": 0,
            "extra": true,
        }),
        references: vec![],
    };
    assert!(malformed_marker
        .decode_typed::<serde_json::Value>()
        .is_err());

    let orphan = SceneComponentAssetRecord {
        type_id: "tests.Component".into(),
        schema_id: "tests.Component.v1".into(),
        schema_version: 1,
        provider_id: "tests".into(),
        payload: serde_json::json!({"value": true}),
        references: vec![AssetReference::from_locator(
            AssetUri::parse("builtin://orphan").unwrap(),
        )],
    };
    assert!(orphan.validate_references().is_err());

    let repeated = SceneComponentAssetRecord {
        type_id: "tests.Component".into(),
        schema_id: "tests.Component.v1".into(),
        schema_version: 1,
        provider_id: "tests".into(),
        payload: serde_json::json!({
            "first": {"$zircon_scene_asset_reference": 0},
            "second": {"$zircon_scene_asset_reference": 0},
        }),
        references: vec![AssetReference::from_locator(
            AssetUri::parse("builtin://repeated").unwrap(),
        )],
    };
    assert!(repeated.validate_references().is_err());

    let non_integer = SceneComponentAssetRecord {
        type_id: "tests.Component".into(),
        schema_id: "tests.Component.v1".into(),
        schema_version: 1,
        provider_id: "tests".into(),
        payload: serde_json::json!({
            "value": {"$zircon_scene_asset_reference": "zero"},
        }),
        references: vec![AssetReference::from_locator(
            AssetUri::parse("builtin://non-integer").unwrap(),
        )],
    };
    assert!(non_integer.validate_references().is_err());
}
