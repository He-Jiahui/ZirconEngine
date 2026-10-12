use super::runtime_owner_key;

#[test]
fn exact_runtime_owner_key_preserves_identity() {
    assert_eq!(runtime_owner_key("rendering"), "rendering.runtime");
}
