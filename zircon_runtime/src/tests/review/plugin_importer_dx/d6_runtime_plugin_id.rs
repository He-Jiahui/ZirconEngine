//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
#[test]
fn review_d6_runtime_plugin_id_accepts_external_string_keys() {
    let plugin_id_source = include_str!("../../../builtin/runtime_modules/ids/plugin_id.rs");
    let plugin_id_tests = include_str!("../../../builtin/runtime_modules/ids/tests/plugin_id.rs");
    let normalized = plugin_id_source.lines().collect::<Vec<_>>().join("\n");
    assert!(normalized.contains("#[cfg(test)]\n#[path = \"tests/plugin_id.rs\"]\nmod tests;"));
    assert_contains_all(
        "mounted PluginId behavior tests",
        plugin_id_tests,
        &[
            "fn runtime_plugin_id_accepts_external_keys_without_core_variant",
            "fn dynamic_plugin_id_storage_retires_with_the_last_generation_owner",
        ],
    );
    let loader_source = include_str!("../../../builtin/runtime_modules/plugin_modules/loader.rs");
    let review_findings = crate::runtime_test_history_concat!(
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../docs/architecture/runtime-architecture-review-m0.md")
    );

    assert_contains_all(
        "RuntimePluginId string-newtype source",
        plugin_id_source,
        &[
            "pub struct RuntimePluginId(RuntimePluginIdStorage)",
            "enum RuntimePluginIdStorage",
            "Static(&'static str)",
            "Dynamic(Arc<str>)",
            "pub const fn from_static",
            "pub fn parse_key",
            "_ => Self(RuntimePluginIdStorage::Dynamic(Arc::from(",
        ],
    );
    assert!(
        !plugin_id_source.contains("pub enum RuntimePluginId"),
        "RuntimePluginId should not regress to a closed public enum"
    );
    assert!(!plugin_id_source.contains(concat!("Box", "::leak")));
    assert!(!plugin_id_source.contains(concat!("static INTERNED", "_KEYS")));

    assert_contains_all(
        "RuntimePluginId external fallback tests",
        plugin_id_tests,
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

#[test]
fn runtime_plugin_id_non_copy_contract_reaches_workspace_consumers() {
    let entry_config_source =
        include_str!("../../../../../zircon_app/src/entry/product_host_config/request.rs");
    let editor_catalog_source =
        include_str!("../../../../../zircon_plugins/first_party_editor_catalog/src/catalog.rs");
    let runtime_catalog_source =
        include_str!("../../../../../zircon_plugins/first_party_runtime_catalog/src/lib.rs");

    let resolver = include_str!("../../../core/framework/project/plugin_selection_resolution.rs");
    assert_contains_all(
        "both catalogs use the shared generation-owned dedup resolver",
        resolver,
        &[
            "pub fn resolve_plugin_selections",
            "if !seen.insert(plugin_id.clone())",
            "match provider(&plugin_id)",
            "PluginSelectionResolutionStatus::Duplicate",
        ],
    );
    assert!(
        !entry_config_source.contains(".copied()"),
        "RuntimePluginId slices must clone generation-owned IDs instead of requiring Copy"
    );
    assert!(
        editor_catalog_source.contains("*_plugin_id == RuntimePluginId::Navigation")
            && editor_catalog_source
                .contains("resolve_plugin_selections(target_mode, manifest, |plugin_id|")
            && !editor_catalog_source.contains("match plugin_id")
            && !editor_catalog_source.contains("match _plugin_id"),
        "editor provider projection must clone the dedup owner and use typed ID equality"
    );
    assert!(
        runtime_catalog_source.contains("_id == RuntimePluginId::Ai")
            && runtime_catalog_source
                .contains("resolve_plugin_selections(target_mode, manifest, |runtime_id|")
            && !runtime_catalog_source.contains("match id")
            && !runtime_catalog_source.contains("match _id"),
        "runtime provider projection must clone the dedup owner and use typed ID equality"
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
