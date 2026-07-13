use crate::core::asset::AssetToolkitOpenRoute;
use crate::core::editor_operation::EditorOperationPath;
use zircon_runtime::asset::AssetUri;

#[test]
fn toolkit_open_route_roundtrips_canonical_locator_without_a_physical_path() {
    let route = AssetToolkitOpenRoute::new(
        AssetUri::parse("res://animation/hero.sequence.zranim").unwrap(),
        EditorOperationPath::parse("animation.sequence.open").unwrap(),
    );

    let payload = serde_json::to_value(&route).unwrap();
    assert_eq!(
        payload,
        serde_json::json!({
            "asset_locator": "res://animation/hero.sequence.zranim",
            "open_operation": "animation.sequence.open",
        })
    );
    assert!(payload.get("path").is_none());

    let restored: AssetToolkitOpenRoute = serde_json::from_value(payload).unwrap();
    assert_eq!(restored, route);
    assert_eq!(
        restored.asset_locator(),
        &AssetUri::parse("res://animation/hero.sequence.zranim").unwrap()
    );
    assert_eq!(
        restored.open_operation(),
        &EditorOperationPath::parse("animation.sequence.open").unwrap()
    );
}

#[test]
fn toolkit_open_route_rejects_the_retired_untyped_path_payload() {
    let error = serde_json::from_value::<AssetToolkitOpenRoute>(serde_json::json!({
        "path": "E:/temp/hero.sequence.zranim",
        "operation_id": "animation.sequence.open",
    }))
    .unwrap_err();

    assert!(error.to_string().contains("asset_locator"));
}

#[test]
fn toolkit_open_route_rejects_an_invalid_operation_path_on_deserialize() {
    let error = serde_json::from_value::<AssetToolkitOpenRoute>(serde_json::json!({
        "asset_locator": "res://animation/hero.sequence.zranim",
        "open_operation": "Invalid Operation",
    }))
    .unwrap_err();

    assert!(error.to_string().contains("editor operation path"));
}
