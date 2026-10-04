use super::*;
use serde_json::json;

#[test]
fn astra_m11_dynamic_containers_preserve_mixed_json_leaves() {
    for (expected, input) in [
        ("List", json!([1, null, {"nested": [true, "value"]}])),
        ("Map", json!({"z": null, "a": [1, {"enabled": false}]})),
        ("List<Map>", json!([{"nested": [1, false]}])),
    ] {
        let value = reflected_value_from_json("game::State", "payload", expected, &input)
            .expect("dynamic container should convert");
        let declared = DeclaredValueType::parse(expected).unwrap();
        assert!(declared.matches_reflected(&value));
        assert_eq!(json_value_from_reflected(value).unwrap(), input);
    }
}

#[test]
fn astra_m11_dynamic_containers_reject_wrong_shapes_and_named_types() {
    for (expected, input) in [
        ("List", json!({})),
        ("Map", json!([])),
        ("List<Map>", json!([42])),
        ("game::State", json!({"payload": 42})),
    ] {
        let error = reflected_value_from_json("game::Owner", "payload", expected, &input)
            .expect_err("mismatched or unresolved declared type should fail");
        assert!(matches!(error, ReflectError::TypeMismatch { .. }));
    }
}
