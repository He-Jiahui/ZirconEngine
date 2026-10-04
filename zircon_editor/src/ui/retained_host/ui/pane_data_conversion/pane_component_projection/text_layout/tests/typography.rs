use super::*;

#[test]
fn projected_typography_preserves_explicit_shrink_overflow() {
    let attributes = BTreeMap::from([(
        "overflow".to_string(),
        toml::Value::String("shrink".to_string()),
    )]);

    assert_eq!(
        projected_typography(&attributes, "button").overflow,
        "shrink"
    );
}
