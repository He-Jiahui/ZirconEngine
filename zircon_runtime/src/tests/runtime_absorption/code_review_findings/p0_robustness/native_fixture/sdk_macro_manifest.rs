#[test]
fn review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest() {
    let fixture = include_str!(
        "../../../../../../../zircon_plugins/native_dynamic_fixture/native/src/lib.rs"
    );
    let plugin_toml =
        include_str!("../../../../../../../zircon_plugins/native_dynamic_fixture/plugin.toml");
    let native_cargo = include_str!(
        "../../../../../../../zircon_plugins/native_dynamic_fixture/native/Cargo.toml"
    );
    let sdk_dist = include_str!("../../../../../../../zircon_plugins/plugin_sdk/src/dist.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md"),
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md")
    );
    let native_fixture_record = review_findings
        .lines()
        .find(|line| {
            line.starts_with(
                "| 2026-06-28 | Plugins 13 native_dynamic_fixture 验证插件合法性审查 |",
            )
        })
        .expect("native_dynamic_fixture review completion record should exist");

    for required in [
        "zircon_plugin_sdk::native_dist_plugin_v3!",
        "package_manifest: PLUGIN_MANIFEST",
        "runtime_entry: zircon_native_dynamic_fixture_runtime_entry_v3",
        "editor_entry: zircon_native_dynamic_fixture_editor_entry_v3",
        "invoke_command: Some(fixture_invoke_command)",
        "native::catch_native_callback_panic(STATUS_PANIC_DIAGNOSTICS",
        "owned_bytes(response)",
    ] {
        assert!(
            fixture.contains(required),
            "native_dynamic_fixture should keep SDK macro-backed ABI owner `{required}`"
        );
    }
    assert!(
        fixture.contains(
            "const PLUGIN_MANIFEST: &str = concat!(include_str!(\"../../plugin.toml\"), \"\\0\");"
        ),
        "native_dynamic_fixture should embed plugin.toml as the single manifest source"
    );
    for stale_manual_abi in [
        "#[no_mangle]",
        "NativePluginDescriptorV3 {",
        "#[repr(C)]",
        "zircon_native_plugin_descriptor_v3(",
    ] {
        assert!(
            !fixture.contains(stale_manual_abi),
            "native_dynamic_fixture should not hand-write native ABI surface `{stale_manual_abi}`"
        );
    }
    assert!(
        native_cargo.contains(
            "zircon_plugin_sdk = { workspace = true, default-features = false, features = [\"native\"] }"
        ),
        "native fixture should consume the SDK native feature instead of defining ABI structs locally"
    );
    assert!(
        sdk_dist.contains("macro_rules! native_dist_plugin_v3")
            && sdk_dist.contains("pub use crate::{")
            && sdk_dist.contains("native_dist_editor_plugin_v3, native_dist_plugin_v3"),
        "plugin SDK should own and export the native dist macro surface"
    );

    for required in [
        "id = \"native_dynamic_fixture\"",
        "description = \"Real dynamic library fixture for ABI v3 native plugin loading with ABI v2 fallback coverage.\"",
        "descriptor_symbol = \"zircon_native_plugin_descriptor_v3\"",
        "runtime_entry = \"zircon_native_dynamic_fixture_runtime_entry_v3\"",
        "editor_entry = \"zircon_native_dynamic_fixture_editor_entry_v3\"",
        "\"runtime.plugin.native_dynamic_fixture\"",
        "\"runtime.asset.importer.native_dynamic_fixture.data_json\"",
        "\"editor.extension.native_dynamic_fixture\"",
    ] {
        assert!(
            plugin_toml.contains(required),
            "plugin.toml should keep native fixture manifest source `{required}`"
        );
    }

    let ds8_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D-S8 |"))
        .expect("D-S8 row should exist");
    let d3_row = review_findings
        .lines()
        .find(|line| line.starts_with("| D3 |"))
        .expect("D3 row should exist");
    for required in [
        "native 插件 ABI v3 样板已由 plugin SDK macro 承接",
        "zircon_plugin_sdk::native_dist_plugin_v3!",
        "native_dynamic_fixture_validation_plugin_review_passed_unused_import_warning_fixed",
        "native_dynamic_fixture_importer_manifest_self_description_static_passed_cargo_deferred",
        "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
        "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            review_findings.contains(required),
            "D-S8 numbered review evidence should record current SDK macro state `{required}`"
        );
    }
    for required in [
        "native manifest 双写已由 plugin.toml 单源闭合",
        "concat!(include_str!(\"../../plugin.toml\"), \"\\0\")",
        "native_dynamic_fixture_validation_plugin_review_passed_unused_import_warning_fixed",
        "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
        "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            review_findings.contains(required),
            "D3 numbered review evidence should record current single-manifest state `{required}`"
        );
    }
    for stale_text in [
        "native 插件零 SDK，手写 ~720 行 ABI v3",
        "native manifest 双写且已漂移",
        "native_dynamic_fixture/plugin.toml:6` vs `native/src/lib.rs:21-48",
    ] {
        assert!(
            !ds8_row.contains(stale_text) && !d3_row.contains(stale_text),
            "D-S8/D3 top rows should not keep stale unresolved text `{stale_text}`"
        );
    }
    assert!(
        ds8_row.ends_with("| Plugins 13 M2 + Plugins 12 / closed |"),
        "D-S8 row should mark native fixture SDK macro convergence closed"
    );
    assert!(
        d3_row.ends_with("| Plugins 13 M1 + Plugins 12 / closed |"),
        "D3 row should mark native fixture single-manifest convergence closed"
    );
    for doc_anchor in [
        "D-S8/D3 native dynamic fixture SDK macro and manifest single source top-table sync",
        "D-S8/D3 native fixture top-row closed status sync",
        "native_dynamic_fixture_validation_plugin_review_passed_unused_import_warning_fixed",
        "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
        "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
    ] {
        assert!(
            review_findings.contains(doc_anchor) || native_fixture_record.contains(doc_anchor),
            "D-S8/D3 review docs should record `{doc_anchor}`"
        );
    }
}
