#[test]
fn review_d6_runtime_plugin_id_accepts_external_string_keys() {
    let plugin_id_source = include_str!(
        "../../../../../../zircon_runtime/src/builtin/runtime_modules/ids/plugin_id.rs"
    );
    let loader_source = include_str!(
        "../../../../../../zircon_runtime/src/builtin/runtime_modules/plugin_modules/loader.rs"
    );
    let review_findings = concat!(
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
    );

    assert_contains_all(
        "RuntimePluginId string-newtype source",
        plugin_id_source,
        &[
            "pub struct RuntimePluginId(&'static str)",
            "pub const fn from_static",
            "pub fn parse_key",
            "_ => Self(intern_runtime_plugin_key(normalized))",
            "fn intern_runtime_plugin_key",
            "runtime_plugin_id_accepts_external_keys_without_core_variant",
        ],
    );
    assert!(
        !plugin_id_source.contains("enum RuntimePluginId"),
        "RuntimePluginId should not regress to a closed enum"
    );

    assert_contains_all(
        "RuntimePluginId external fallback tests",
        plugin_id_source,
        &[
            r#"RuntimePluginId::new("third_party.weather_sim")"#,
            r#""third_party.weather_sim""#,
            r#""Third_Party.Weather_Sim""#,
            r#"".starts_with_dot""#,
            r#""bad/id""#,
        ],
    );
    assert_contains_all(
        "runtime plugin loader source",
        loader_source,
        &[
            "RuntimePluginId",
            "if id != RuntimePluginId::Ui",
            "return None",
        ],
    );
    assert!(
        !loader_source.contains("externalized_runtime_plugin_module"),
        "external plugin loading must stay registry-owned instead of returning a builtin warning shim"
    );

    let d6_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D6 |"))
        .expect("D6 review finding row should exist");
    assert_contains_all(
        "D6 review row",
        review_findings,
        &[
            "RuntimePluginId",
            "开放 string-newtype",
            "第三方合法 key 不需 core enum 分支",
            "d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred",
            "review_d6_runtime_plugin_id_accepts_external_string_keys",
        ],
    );
    assert!(
        d6_row.ends_with("| M5 / 跨计划（已关闭） |"),
        "D6 row should retain its closed cross-plan status"
    );

    assert_contains_all(
        "D6 numbered review output",
        review_findings,
        &[
            "Runtime 15 M3 D6 RuntimePluginId open string-newtype review sync",
            "d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs",
            "review_d6_runtime_plugin_id_accepts_external_string_keys",
            "RuntimePluginId",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, anchors: &[&str]) {
    let missing: Vec<&str> = anchors
        .iter()
        .copied()
        .filter(|anchor| !source.contains(anchor))
        .collect();
    assert!(missing.is_empty(), "{label} missing anchors: {missing:?}");
}
