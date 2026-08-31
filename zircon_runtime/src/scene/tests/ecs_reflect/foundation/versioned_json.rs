use zircon_runtime_interface::reflect::ReflectValueFloatKind;
use zircon_runtime_interface::serialization::LoadError;

use super::*;
use crate::scene::tests::authoring_boundary::{
    assert_text_excludes_authoring_tokens, SERIALIZED_AUTHORING_TOKENS,
};
use crate::scene::ReflectedJsonError;

const V0_REFLECTED_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../tests/fixtures/serialization/scene-reflection/v0/reflected-value.json"
));

#[test]
fn reflected_json_v0_migrates_asset_refs_and_resaves_idempotently() {
    let loaded = reflected_from_json(V0_REFLECTED_JSON).expect("v0 fixture should migrate");
    assert_eq!(loaded.migrated_from, Some(0));
    let ReflectedValue::Json(value) = &loaded.value else {
        panic!("unwrapped v0 JSON must migrate to the reflected Json variant");
    };
    assert_eq!(
        value["material"],
        json!({
            "guid": "3d6f0a55-a138-4cab-a54a-1c8c7262d101",
            "kind": "project",
            "path_hint": "assets/materials/hero.zmaterial",
            "sub": "surface"
        })
    );
    assert!(value["material"].get("uuid").is_none());
    assert!(value["material"].get("url").is_none());

    let first = json_from_reflected(&loaded.value).expect("migrated value should save");
    assert_text_excludes_authoring_tokens(
        "versioned reflected JSON",
        &first,
        SERIALIZED_AUTHORING_TOKENS,
    );
    let current = reflected_from_json(&first).expect("current value should reload");
    assert_eq!(current.migrated_from, None);
    assert_eq!(current.value, loaded.value);
    let second = json_from_reflected(&current.value).expect("current value should resave");
    assert_eq!(second.as_bytes(), first.as_bytes());

    let envelope: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(
        envelope["$zircon"]["header"],
        json!({
            "schema_id": "zircon.scene.reflected-json",
            "schema_version": 1
        })
    );
}

#[test]
fn retired_asset_ref_migration_only_rewrites_the_exact_retired_shape() {
    for unchanged in [
        json!({
            "uuid": "3d6f0a55-a138-4cab-a54a-1c8c7262d101",
            "url": "assets/materials/hero.zmaterial#surface",
            "extra": true
        }),
        json!({
            "uuid": "3d6f0a55-a138-4cab-a54a-1c8c7262d101"
        }),
        json!({
            "guid": "3d6f0a55-a138-4cab-a54a-1c8c7262d101",
            "path_hint": "assets/materials/hero.zmaterial",
            "sub": "surface"
        }),
    ] {
        let loaded = reflected_from_json(&unchanged.to_string())
            .expect("non-exact retired asset references should remain unchanged");
        assert_eq!(loaded.value, ReflectedValue::Json(unchanged));
    }
}

#[test]
fn reflected_json_rejects_future_headers_before_payload_decode() {
    let future = r#"{
      "$zircon": {
        "header": {
          "schema_id": "zircon.scene.reflected-json",
          "schema_version": 2
        },
        "payload": { "not": "the current DTO" }
      }
    }"#;
    let error = reflected_from_json(future).expect_err("future input must be refused");
    assert!(matches!(
        error,
        ReflectedJsonError::Load(LoadError::FutureVersion {
            found: 2,
            supported: 1,
            ..
        })
    ));
}

#[test]
fn reflected_json_writer_rejects_non_finite_values_with_typed_source() {
    let error = json_from_reflected(&ReflectedValue::List(vec![ReflectedValue::Vec2([
        1.0,
        f32::NAN,
    ])]))
    .expect_err("non-finite reflected values cannot be JSON");
    assert!(matches!(
        error,
        ReflectedJsonError::Value(
            zircon_runtime_interface::reflect::ReflectValueValidationError::NonFiniteFloat {
                kind: ReflectValueFloatKind::Vec2,
                component: 1,
            }
        )
    ));
}
