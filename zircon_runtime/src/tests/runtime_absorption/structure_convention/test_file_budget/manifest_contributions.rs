use super::*;

#[test]
fn runtime_15_manifest_contributions_tests_are_folder_backed() {
    let parent = read_runtime_src("tests/plugin_extensions/manifest_contributions.rs");
    let editor_only =
        read_runtime_src("tests/plugin_extensions/manifest_contributions/editor_only.rs");
    let net = read_runtime_src("tests/plugin_extensions/manifest_contributions/net.rs");
    let runtime_family =
        read_runtime_src("tests/plugin_extensions/manifest_contributions/runtime_family.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let package_manifest_doc = read_repo("docs/zircon_runtime/plugin/package_manifest.md");

    assert_contains_all(
        "manifest contributions parent mounts child owners",
        &parent,
        &[
            "#[path = \"manifest_contributions/editor_only.rs\"]",
            "mod editor_only;",
            "#[path = \"manifest_contributions/net.rs\"]",
            "mod net;",
            "#[path = \"manifest_contributions/runtime_family.rs\"]",
            "mod runtime_family;",
        ],
    );

    for moved_test in [
        "fn editor_only_plugin_tomls_declare_package_level_targets_and_capabilities",
        "fn low_overlap_editor_only_plugin_tomls_declare_explicit_experimental_maturity",
        "fn net_plugin_toml_declares_content_download_http_dependency",
        "fn builtin_net_catalog_declares_layered_optional_features",
        "fn sound_plugin_manifest_matches_catalog_beta_partial_metadata",
        "fn particles_plugin_toml_matches_catalog_optional_feature_metadata",
        "fn runtime_experimental_plugin_toml_matches_catalog_partial_metadata",
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
    assert_contains_all(
        "runtime-family child owns non-rendering runtime manifest contracts",
        &runtime_family,
        &[
            "use super::*;",
            "fn sound_plugin_manifest_matches_catalog_beta_partial_metadata",
            "fn animation_plugin_toml_matches_catalog_beta_partial_metadata",
            "fn navigation_plugin_toml_matches_catalog_beta_partial_metadata",
            "fn particles_plugin_toml_matches_catalog_optional_feature_metadata",
            "fn texture_plugin_manifest_matches_catalog_stable_complete_metadata",
            "fn runtime_experimental_plugin_toml_matches_catalog_partial_metadata",
        ],
    );

    let moved_test_count = [editor_only.as_str(), net.as_str(), runtime_family.as_str()]
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
        (
            "tests/plugin_extensions/manifest_contributions/runtime_family.rs",
            runtime_family.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
