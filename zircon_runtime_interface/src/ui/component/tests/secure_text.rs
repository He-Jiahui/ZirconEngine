use super::*;

#[test]
fn serialized_reference_contains_identity_but_no_text_payload() {
    let reference =
        UiSecureTextValueRef::issue(UiTreeId::new("secure.tree"), UiNodeId::new(7), "value_text");

    let json = serde_json::to_string(&reference).unwrap();
    assert!(json.contains("secure.tree"));
    assert!(json.contains("value_text"));
    assert!(!json.contains("password"));
    assert_eq!(
        serde_json::from_str::<UiSecureTextValueRef>(&json).unwrap(),
        reference
    );
    assert!(!format!("{reference:?}").contains(&reference.token.to_string()));
}
