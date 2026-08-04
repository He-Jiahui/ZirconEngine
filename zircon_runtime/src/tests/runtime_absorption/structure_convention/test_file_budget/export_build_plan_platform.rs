use super::*;

#[test]
fn runtime_15_export_build_plan_platform_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/export_build_plan_platform.rs");
    let browser_hosts =
        read_runtime_src("tests/plugin_extensions/export_build_plan_platform/browser_hosts.rs");
    let release_adapters =
        read_runtime_src("tests/plugin_extensions/export_build_plan_platform/release_adapters.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let export_build_plan_doc = read_repo("docs/zircon_runtime/plugin/export_build_plan.md");

    assert_contains_all(
        "export build plan platform parent mounts browser host child owner",
        &parent,
        &[
            "#[path = \"export_build_plan_platform/browser_hosts.rs\"]",
            "mod browser_hosts;",
            "#[path = \"export_build_plan_platform/release_adapters.rs\"]",
            "mod release_adapters;",
        ],
    );

    for moved_test in [
        "fn generated_browser_hosts_instantiate_wasm_exports_and_gate_asset_origins",
        "fn source_template_emits_signing_and_cdn_release_contracts",
        "fn generated_release_adapters_gate_real_store_and_cdn_upload_inputs",
    ] {
        assert!(
            !parent.contains(moved_test),
            "export build plan platform parent should mount child owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "browser host child owns WASM export and asset-origin contracts",
        &browser_hosts,
        &[
            "use super::*;",
            "fn generated_browser_hosts_instantiate_wasm_exports_and_gate_asset_origins",
            "WebAssembly.instantiateStreaming(fetch(manifest.wasmModule), zirconExportImports)",
            "\\\"allowedAssetRoot\\\": \\\"./assets/\\\"",
        ],
    );

    assert_eq!(
        release_adapters.matches("#[test]").count(),
        2,
        "release adapters child should own signing/CDN release contracts"
    );

    assert_contains_all(
        "release adapters child owns signing, store, and CDN upload contracts",
        &release_adapters,
        &[
            "use super::*;",
            "fn source_template_emits_signing_and_cdn_release_contracts",
            "fn generated_release_adapters_gate_real_store_and_cdn_upload_inputs",
            "ZR_GOOGLE_PLAY_SERVICE_ACCOUNT_JSON",
            "ZR_APP_STORE_CONNECT_PRIVATE_KEY_PATH",
            "ZR_CDN_UPLOAD_COMMAND",
        ],
    );

    assert_eq!(
        parent.matches("#[test]").count()
            + browser_hosts.matches("#[test]").count()
            + release_adapters.matches("#[test]").count(),
        10,
        "export build plan platform parent plus split children should preserve the original 10 tests"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/export_build_plan_platform.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/export_build_plan_platform/browser_hosts.rs",
            browser_hosts.as_str(),
        ),
        (
            "tests/plugin_extensions/export_build_plan_platform/release_adapters.rs",
            release_adapters.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
