use crate::project::{AssetRef, AssetRefError, RelPath};
use crate::resource::AssetUuid;
use std::collections::BTreeSet;

fn guid() -> AssetUuid {
    AssetUuid::from_stable_label("project-asset-ref-test")
}

#[test]
fn asset_ref_json_roundtrip_preserves_guid_path_hint_and_subasset() {
    let reference = AssetRef::try_new(
        guid(),
        RelPath::parse("assets/models/hero.glb").unwrap(),
        Some("Mesh0/Primitive1".to_string()),
    )
    .unwrap();

    let encoded = serde_json::to_string(&reference).unwrap();
    let decoded: AssetRef = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, reference);
    assert_eq!(decoded.guid(), guid());
    assert_eq!(decoded.path_hint().as_str(), "assets/models/hero.glb");
    assert_eq!(decoded.sub(), Some("Mesh0/Primitive1"));
}

#[test]
fn asset_ref_human_readable_shape_has_exact_keys_and_rejects_unknown_fields() {
    let reference = AssetRef::try_new(
        guid(),
        RelPath::parse("assets/models/hero.glb").unwrap(),
        Some("Mesh0".to_string()),
    )
    .unwrap();
    let value = serde_json::to_value(reference).unwrap();
    let keys = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    assert_eq!(keys, BTreeSet::from(["guid", "path_hint", "sub"]));

    let mut unknown = value;
    unknown["url"] = serde_json::json!("res://models/hero.glb");
    assert!(serde_json::from_value::<AssetRef>(unknown).is_err());
}

#[test]
fn asset_ref_bincode_roundtrip_is_deterministic() {
    let reference = AssetRef::try_new(
        "11111111-2222-4333-8444-555555555555".parse().unwrap(),
        RelPath::parse("assets/models/hero.glb").unwrap(),
        Some("Mesh0/Primitive1".to_string()),
    )
    .unwrap();

    const HISTORICAL_V1: &[u8] = &[
        0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x11, 0x11, 0x11, 0x22, 0x22, 0x43,
        0x33, 0x84, 0x44, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x16, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, b'a', b's', b's', b'e', b't', b's', b'/', b'm', b'o', b'd', b'e', b'l', b's',
        b'/', b'h', b'e', b'r', b'o', b'.', b'g', b'l', b'b', 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, b'M', b'e', b's', b'h', b'0', b'/', b'P', b'r', b'i', b'm', b'i', b't',
        b'i', b'v', b'e', b'1',
    ];

    let encoded = bincode::serialize(&reference).unwrap();
    let decoded: AssetRef = bincode::deserialize(HISTORICAL_V1).unwrap();

    assert_eq!(encoded, HISTORICAL_V1);
    assert_eq!(decoded, reference);
}

#[test]
fn asset_ref_without_subasset_keeps_none_semantics() {
    let reference = AssetRef::try_new(
        guid(),
        RelPath::parse("assets/textures/hero.png").unwrap(),
        None,
    )
    .unwrap();

    assert_eq!(reference.sub(), None);
    assert!(serde_json::to_value(reference).unwrap()["sub"].is_null());
}

#[test]
fn asset_ref_constructor_rejects_invalid_subasset_paths() {
    for (sub, expected) in [
        ("", AssetRefError::EmptySubPath),
        ("Mesh#0", AssetRefError::FragmentDelimiterInSubPath),
        (
            "Mesh\n0",
            AssetRefError::ControlCharacterInSubPath { index: 4 },
        ),
    ] {
        let error = AssetRef::try_new(
            guid(),
            RelPath::parse("assets/models/hero.glb").unwrap(),
            Some(sub.to_string()),
        )
        .unwrap_err();

        assert_eq!(error, expected, "accepted invalid subasset path {sub:?}");
    }
}

#[test]
fn asset_ref_deserialization_rejects_invalid_subasset_paths() {
    for sub in ["", "Mesh#0", "Mesh\u{0007}0"] {
        let encoded = serde_json::json!({
            "guid": guid(),
            "path_hint": "assets/models/hero.glb",
            "sub": sub,
        });

        assert!(
            serde_json::from_value::<AssetRef>(encoded).is_err(),
            "deserialized invalid subasset path {sub:?}"
        );
    }
}

#[test]
fn asset_ref_deserialization_rejects_path_hint_traversal() {
    let encoded = serde_json::json!({
        "guid": guid(),
        "path_hint": "assets/../outside/hero.glb",
        "sub": null,
    });

    assert!(serde_json::from_value::<AssetRef>(encoded).is_err());
}
