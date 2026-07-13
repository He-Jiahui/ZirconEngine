use crate::core::asset::{AssetTypeId, AssetTypeIdError};

#[test]
fn asset_type_id_accepts_canonical_open_plugin_keys() {
    for value in [
        "model",
        "material.graph",
        "support.asset",
        "terrain.heightfield",
        "tilemap_2d.tilemap",
    ] {
        let id = AssetTypeId::parse(value).expect("canonical asset type id");
        assert_eq!(id.as_str(), value);
        assert_eq!(id.to_string(), value);
    }
}

#[test]
fn asset_type_id_rejects_non_canonical_or_path_like_keys() {
    for value in [
        "",
        "Model",
        ".model",
        "model.",
        "material..graph",
        "material graph",
        "material/graph",
        "material\\graph",
        "1model",
        "material.-graph",
        "material\ngraph",
    ] {
        assert!(
            matches!(
                AssetTypeId::parse(value),
                Err(AssetTypeIdError::InvalidAssetTypeId { .. })
            ),
            "`{value:?}` must be rejected"
        );
    }
}

#[test]
fn asset_type_id_serde_roundtrip_keeps_the_validated_newtype() {
    let id = AssetTypeId::parse("animation.state_machine").unwrap();
    let encoded = serde_json::to_string(&id).unwrap();
    assert_eq!(encoded, "\"animation.state_machine\"");
    assert_eq!(serde_json::from_str::<AssetTypeId>(&encoded).unwrap(), id);
    assert!(serde_json::from_str::<AssetTypeId>("\"Animation.StateMachine\"").is_err());
}
