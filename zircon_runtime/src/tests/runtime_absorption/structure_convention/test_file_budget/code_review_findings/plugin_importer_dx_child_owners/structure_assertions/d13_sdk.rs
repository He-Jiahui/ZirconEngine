use super::super::super::super::*;

const PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions.rs";
const PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/structure_assertions/d13_sdk.rs";
const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;

pub(super) fn assert_plugin_importer_d13_sdk_child_owners_are_folder_backed() {
    let plugin_importer_dx_d13 = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
    );
    let plugin_importer_dx_d13_manifest_parity = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/manifest_parity.rs",
    );
    let plugin_importer_dx_d13_runtime_crates = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_crates.rs",
    );
    let plugin_importer_dx_d13_runtime_exports = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs",
    );
    let plugin_importer_dx_d13_runtime_manifests = read_runtime_src(
        "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_manifests.rs",
    );

    assert_contains_all(
        "plugin importer DX D13 parent mounts focused SDK review guard children",
        &plugin_importer_dx_d13,
        &[
            "#[path = \"d13_importer_sdk/runtime_crates.rs\"]",
            "mod runtime_crates;",
            "#[path = \"d13_importer_sdk/runtime_exports.rs\"]",
            "mod runtime_exports;",
            "#[path = \"d13_importer_sdk/runtime_manifests.rs\"]",
            "mod runtime_manifests;",
            "#[path = \"d13_importer_sdk/manifest_parity.rs\"]",
            "mod manifest_parity;",
        ],
    );
    assert_eq!(
        plugin_importer_dx_d13.matches("#[test]").count(),
        0,
        "plugin_importer_dx/d13_importer_sdk.rs should only mount child review guard owners"
    );
    for child_owned_test in [
        "fn review_d13_importer_runtime_exports_use_sdk_macro",
        "fn review_d13_importer_runtime_manifests_use_sdk_builder",
        "fn review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
    ] {
        assert!(
            !plugin_importer_dx_d13.contains(child_owned_test),
            "child-owned D13 importer SDK review guard `{child_owned_test}` should not return to plugin_importer_dx/d13_importer_sdk.rs"
        );
    }
    assert_contains_all(
        "plugin importer DX D13 runtime-crates child owns importer runtime source inventory",
        &plugin_importer_dx_d13_runtime_crates,
        &[
            "pub(super) const IMPORTER_RUNTIME_CRATES",
            "asset_importers/audio",
            "asset_importers/data",
            "asset_importers/model",
            "asset_importers/shader",
            "asset_importers/texture",
            "ui_document_importer",
        ],
    );
    assert_contains_all(
        "plugin importer DX D13 runtime-exports child owns export macro review guard",
        &plugin_importer_dx_d13_runtime_exports,
        &[
            "fn review_d13_importer_runtime_exports_use_sdk_macro",
            "d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred",
            "zircon_plugin_sdk::runtime_plugin_exports!",
        ],
    );
    assert_contains_all(
        "plugin importer DX D13 runtime-manifests child owns manifest builder review guard",
        &plugin_importer_dx_d13_runtime_manifests,
        &[
            "fn review_d13_importer_runtime_manifests_use_sdk_builder",
            "d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred",
            "ImporterRuntimeManifestBuilder",
        ],
    );
    assert_contains_all(
        "plugin importer DX D13 manifest-parity child owns parity review guard",
        &plugin_importer_dx_d13_manifest_parity,
        &[
            "fn review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
            "d13_importer_manifest_parity_guard_static_passed_cargo_deferred",
            "d13_importer_top_row_closed_status_static_passed_cargo_deferred",
            "importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity",
            "NATIVE_ABI_VERSION_V3",
            "NATIVE_DESCRIPTOR_SYMBOL_V3",
        ],
    );
}

#[test]
fn runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD);
    let child = read_runtime_src(PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD);

    assert_contains_all(
        "plugin-importer DX structure assertions delegate D13 SDK structure checks to child owner",
        &parent,
        &[
            "#[path = \"structure_assertions/d13_sdk.rs\"]",
            "mod d13_sdk;",
            "d13_sdk::assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
        ],
    );
    for moved_anchor in [
        "let plugin_importer_dx_d13 = read_runtime_src(",
        concat!(
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/",
            "d13_importer_sdk/manifest_parity.rs"
        ),
        concat!(
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/",
            "d13_importer_sdk/runtime_exports.rs"
        ),
        concat!("fn review_d13_importer_runtime_exports_", "use_sdk_macro"),
        concat!(
            "fn review_d13_importer_manifest_parity_guard_",
            "lives_in_sdk_builder"
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "plugin_importer_dx_child_owners/structure_assertions.rs should delegate D13 SDK structure assertion anchor `{moved_anchor}` to d13_sdk.rs"
        );
    }
    assert_contains_all(
        "plugin-importer D13 SDK structure assertions child owns focused SDK checks",
        &child,
        &[
            "pub(super) fn assert_plugin_importer_d13_sdk_child_owners_are_folder_backed",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/manifest_parity.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_crates.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_manifests.rs",
            "review_d13_importer_runtime_exports_use_sdk_macro",
            "review_d13_importer_runtime_manifests_use_sdk_builder",
            "review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
            "runtime_15_plugin_importer_d13_sdk_structure_assertions_are_child_owner",
        ],
    );
    assert_plugin_importer_d13_sdk_child_owners_are_folder_backed();

    for (path, source) in [
        (
            PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_CHILD,
            parent.as_str(),
        ),
        (
            PLUGIN_IMPORTER_D13_STRUCTURE_ASSERTIONS_CHILD,
            child.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
