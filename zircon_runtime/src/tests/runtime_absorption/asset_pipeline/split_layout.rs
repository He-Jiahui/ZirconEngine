const PARENT_SOURCE: &str = include_str!("../asset_pipeline.rs");
const INVENTORY_SOURCE: &str = include_str!("inventory.rs");
const MIRROR_DOCS_SOURCE: &str = include_str!("mirror_docs.rs");
const SUPPORT_SOURCE: &str = include_str!("support.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

const RUNTIME_15_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md"
);
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/zircon_runtime/structure/module-convention.md");
const FRAMEWORKS_02_OUTPUT_RECORDS: &str = include_str!(
    "../../../../../docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md"
);

#[test]
fn runtime_15_asset_pipeline_route_owner_is_folder_backed() {
    assert_contains_all(
        "parent route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"asset_pipeline/inventory.rs\"]",
            "#[path = \"asset_pipeline/mirror_docs.rs\"]",
            "#[path = \"asset_pipeline/support.rs\"]",
            "#[path = \"asset_pipeline/split_layout.rs\"]",
        ],
    );
    assert_parent_route_only();
    assert_child_owners_are_focused();
    assert_line_budget();
    assert_docs_and_status_mirror_split();
}

fn assert_parent_route_only() {
    assert!(
        !PARENT_SOURCE.contains("#[test]"),
        "asset_pipeline.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "EXPECTED_RUNTIME_04_BEHAVIOR_TEST_ANCHORS",
        "fn runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts",
        "fn assert_contains_all",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "asset_pipeline.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "inventory child",
        INVENTORY_SOURCE,
        &[
            "EXPECTED_RUNTIME_04_SOURCE_FILES",
            "EXPECTED_RUNTIME_04_GUARD_FILES",
            "EXPECTED_RUNTIME_04_GUARD_ANCHORS",
            "ASSET_PIPELINE_MIRROR_DOC_ANCHORS",
        ],
    );
    assert_contains_all(
        "mirror docs child",
        MIRROR_DOCS_SOURCE,
        &[
            "runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts",
            "EXPECTED_RUNTIME_04_BEHAVIOR_TEST_ANCHORS",
            "ASSET_PIPELINE_MIRROR_DOC_ANCHORS",
        ],
    );
    assert_contains_all(
        "support child",
        SUPPORT_SOURCE,
        &[
            "pub(super) fn assert_files_exist",
            "pub(super) fn assert_contains_all",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 14),
        ("inventory child", INVENTORY_SOURCE, 150),
        ("mirror docs child", MIRROR_DOCS_SOURCE, 150),
        ("support child", SUPPORT_SOURCE, 40),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 220),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    assert!(
        RUNTIME_15_OUTPUT_RECORDS
            .contains("runtime_15_asset_pipeline_route_owner_split_static_passed_cargo_deferred"),
        "Runtime 15 output records should own the asset_pipeline route-owner split status"
    );
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "asset_pipeline/inventory.rs",
            "asset_pipeline/mirror_docs.rs",
            "asset_pipeline/support.rs",
            "asset_pipeline/split_layout.rs",
            "runtime_15_asset_pipeline_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 output records",
        FRAMEWORKS_02_OUTPUT_RECORDS,
        &[
            "frameworks_02_m3_asset_pipeline_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 asset-pipeline route-owner split",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain split anchor `{anchor}`"
        );
    }
}
