use super::wire_runtime_catalog_cargo;

#[test]
fn runtime_catalog_wiring_rejects_an_existing_feature_without_overwriting_it() {
    let source = r#"[features]
base-runtime-plugins = []
demo-probe-runtime-plugin = ["manual-contract"]

[dependencies]
"#;

    let error = wire_runtime_catalog_cargo(source, "demo_probe").unwrap_err();

    assert!(error
        .to_string()
        .contains("runtime catalog feature `demo-probe-runtime-plugin` already exists"));
}
