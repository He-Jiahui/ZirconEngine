use super::parse_bounded_load_manifest;

#[test]
fn bounded_load_manifest_rejects_an_entry_before_unbounded_vector_growth() {
    let source = r#"
[[plugins]]
id = "weather"
path = "plugins/weather"
manifest = "plugins/weather/plugin.toml"

[[plugins]]
id = "climate"
path = "plugins/climate"
manifest = "plugins/climate/plugin.toml"
"#;

    let error = parse_bounded_load_manifest(source, 1)
        .expect_err("second selection entry must exceed the admitted candidate capacity");

    assert!(error
        .to_string()
        .contains("exceeds the admitted candidate budget"));
}
