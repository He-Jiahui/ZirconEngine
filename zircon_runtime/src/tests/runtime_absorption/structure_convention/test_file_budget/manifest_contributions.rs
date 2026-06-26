use super::*;

#[test]
fn runtime_15_manifest_contributions_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/manifest_contributions.rs");
    let editor_only =
        read_runtime_src("tests/plugin_extensions/manifest_contributions/editor_only.rs");
    let net = read_runtime_src("tests/plugin_extensions/manifest_contributions/net.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let package_manifest_doc = read_repo("docs/zircon_runtime/plugin/package_manifest.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/scene_script_tests.rs",
    );

    assert_contains_all(
        "manifest contributions parent mounts child owners",
        &parent,
        &[
            "#[path = \"manifest_contributions/editor_only.rs\"]",
            "mod editor_only;",
            "#[path = \"manifest_contributions/net.rs\"]",
            "mod net;",
        ],
    );

    for moved_test in [
        "fn editor_only_plugin_tomls_declare_package_level_targets_and_capabilities",
        "fn low_overlap_editor_only_plugin_tomls_declare_explicit_experimental_maturity",
        "fn net_plugin_toml_declares_content_download_http_dependency",
        "fn builtin_net_catalog_declares_layered_optional_features",
    ] {
        assert!(
            !parent.contains(moved_test),
            "manifest contributions parent should mount child owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "editor-only child owns editor-only plugin manifest contracts",
        &editor_only,
        &[
            "use super::*;",
            "fn editor_only_plugin_tomls_declare_package_level_targets_and_capabilities",
            "fn low_overlap_editor_only_plugin_tomls_declare_explicit_experimental_maturity",
            "fn editor_authoring_plugin_tomls_declare_explicit_experimental_maturity",
        ],
    );
    assert_contains_all(
        "net child owns net plugin manifest contracts",
        &net,
        &[
            "use super::*;",
            "fn net_plugin_toml_declares_content_download_http_dependency",
            "fn builtin_net_catalog_declares_layered_optional_features",
            "net.content_download",
        ],
    );

    let moved_test_count = [editor_only.as_str(), net.as_str()]
        .iter()
        .map(|source| source.matches("#[test]").count())
        .sum::<usize>();
    assert_eq!(
        parent.matches("#[test]").count() + moved_test_count,
        13,
        "manifest contributions parent plus split children should preserve the original 13 tests"
    );

    for (path, source) in [
        (
            "tests/plugin_extensions/manifest_contributions.rs",
            parent.as_str(),
        ),
        (
            "tests/plugin_extensions/manifest_contributions/editor_only.rs",
            editor_only.as_str(),
        ),
        (
            "tests/plugin_extensions/manifest_contributions/net.rs",
            net.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("package manifest doc", package_manifest_doc.as_str()),
        ("status-output scene/script row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 manifest contributions test folder split",
                "runtime_15_manifest_contributions_tests_folder_split_static_passed_cargo_deferred",
                "tests/plugin_extensions/manifest_contributions.rs",
                "tests/plugin_extensions/manifest_contributions/editor_only.rs",
                "tests/plugin_extensions/manifest_contributions/net.rs",
                "runtime_15_manifest_contributions_tests_are_folder_backed",
            ],
        );
    }
}
