use super::*;

#[test]
fn empty_option_source_ignores_orphan_option_state() {
    let attributes = BTreeMap::from([
        ("query".to_string(), toml::Value::String("build".into())),
        (
            "hovered_option_id".to_string(),
            toml::Value::String("build.project".into()),
        ),
        (
            "selected_options".to_string(),
            toml::Value::Array(vec![toml::Value::String("build.project".into())]),
        ),
    ]);

    assert!(projected_structured_options(&attributes, &[]).is_empty());
}
