use super::super::super::*;

const PLUGIN_IMPORTER_DX_STRUCTURE_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners.rs";
const PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/plugin_importer_dx_child_owners/source_inventory.rs";
const PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET: usize = 800;

const PLUGIN_IMPORTER_DX_SOURCE_PATHS: &[&str] = &[
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d1_capability_single_source.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d11_test_runtime_fixture.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d12_runtime_exports.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/manifest_parity.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_crates.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_manifests.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d5_editor_authoring.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d8_registration_builder.rs",
    "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d9_editor_runtime_mirror.rs",
];

fn plugin_importer_dx_sources() -> Vec<(&'static str, String)> {
    PLUGIN_IMPORTER_DX_SOURCE_PATHS
        .iter()
        .map(|path| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn assert_plugin_importer_dx_line_budgets() {
    for (path, source) in plugin_importer_dx_sources() {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the plugin-importer DX child-owner budget; got {line_count} lines"
        );
    }
}

pub(super) fn plugin_importer_dx_review_guard_count() -> usize {
    plugin_importer_dx_sources()
        .iter()
        .map(|(_, source)| source.matches("#[test]").count())
        .sum()
}

#[test]
fn runtime_15_plugin_importer_dx_source_inventory_is_child_owner() {
    let parent = read_runtime_src(PLUGIN_IMPORTER_DX_STRUCTURE_CHILD);
    let child = read_runtime_src(PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD);

    assert_contains_all(
        "plugin-importer DX structure guard delegates source inventory to child owner",
        &parent,
        &[
            "#[path = \"plugin_importer_dx_child_owners/source_inventory.rs\"]",
            "mod source_inventory;",
            "source_inventory::assert_plugin_importer_dx_line_budgets",
            "source_inventory::plugin_importer_dx_review_guard_count",
        ],
    );
    assert!(
        !parent.contains("const PLUGIN_IMPORTER_DX_SOURCE_PATHS"),
        "plugin_importer_dx_child_owners.rs should not retain the plugin-importer DX source inventory"
    );
    assert!(
        !parent.contains("fn plugin_importer_dx_sources()"),
        "plugin_importer_dx_child_owners.rs should delegate plugin-importer DX source reads to source_inventory.rs"
    );
    assert_contains_all(
        "plugin-importer DX source inventory child owns source paths and count helpers",
        &child,
        &[
            "const PLUGIN_IMPORTER_DX_SOURCE_PATHS",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d10_bridge_call.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk/runtime_exports.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d9_editor_runtime_mirror.rs",
            "pub(super) fn assert_plugin_importer_dx_line_budgets",
            "pub(super) fn plugin_importer_dx_review_guard_count",
        ],
    );
    assert_eq!(
        plugin_importer_dx_review_guard_count(),
        11,
        "plugin-importer DX source inventory should preserve all current D1/D5/D6/D8/D9/D10/D11/D12/D13 review guards"
    );

    for (path, source) in [
        (PLUGIN_IMPORTER_DX_STRUCTURE_CHILD, parent.as_str()),
        (PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_CHILD, child.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < PLUGIN_IMPORTER_DX_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
