//! 脚本宿主绑定、游戏流程和门禁文档需对应同一运行时契约。通过源码文本核对父子路由、状态镜像和文件预算。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
const PARENT_SOURCE: &str = include_str!("../script_binding.rs");
const GAMEPLAY_HOST_SOURCE: &str = include_str!("gameplay_host.rs");
const INVENTORY_SOURCE: &str = include_str!("inventory.rs");
const MIRROR_DOCS_SOURCE: &str = include_str!("mirror_docs.rs");
const CURRENT_INVENTORY_SOURCE: &str = include_str!("mirror_docs/current_inventory.rs");
const SUPPORT_SOURCE: &str = include_str!("support.rs");
const SPLIT_LAYOUT_SOURCE: &str = include_str!("split_layout.rs");

fn runtime_13_plan() -> &'static str {
    history("docs/plans/zircon_runtime/runtime/13/2026-07-09-script-binding-and-reflection-output-records.md")
}
fn runtime_15_plan() -> &'static str {
    crate::tests::runtime_absorption::current_source_fixture::runtime_architecture_implementation_output()
}
fn runtime_index() -> &'static str {
    history(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    )
}
fn structure_convention_plan() -> &'static str {
    history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md")
}
fn review_findings_plan() -> &'static str {
    history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
}
const MODULE_CONVENTION_DOC: &str =
    include_str!("../../../../../docs/crates/zircon_runtime/structure/module-convention.md");
fn frameworks_02_plan() -> &'static str {
    history("docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md")
}

#[test]
fn runtime_15_script_binding_route_owner_is_folder_backed() {
    assert_contains_all(
        "parent route owner",
        PARENT_SOURCE,
        &[
            "#[path = \"script_binding/gameplay_host.rs\"]",
            "#[path = \"script_binding/inventory.rs\"]",
            "#[path = \"script_binding/mirror_docs.rs\"]",
            "#[path = \"script_binding/support.rs\"]",
            "#[path = \"script_binding/split_layout.rs\"]",
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
        "script_binding.rs should route child owners instead of owning tests"
    );
    for forbidden in [
        "EXPECTED_RUNTIME_13_SOURCE_FILES",
        "fn runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
        "fn runtime_13_gameplay_host_owner_split_keeps_domain_files",
        "fn assert_files_exist",
        "fn count_occurrences",
    ] {
        assert!(
            !PARENT_SOURCE.contains(forbidden),
            "script_binding.rs should not retain `{forbidden}`"
        );
    }
}

fn assert_child_owners_are_focused() {
    assert_contains_all(
        "inventory child",
        INVENTORY_SOURCE,
        &[
            "EXPECTED_RUNTIME_13_SOURCE_FILES",
            "EXPECTED_RUNTIME_13_TEST_FILES",
            "GAMEPLAY_HOST_OWNER_FILES",
            "SCRIPT_BINDING_MIRROR_DOC_ANCHORS",
        ],
    );
    assert_contains_all(
        "mirror docs child",
        MIRROR_DOCS_SOURCE,
        &[
            "runtime_13_script_binding_mirror_docs_match_structure_audit_counts",
            "RUNTIME_13_GUARD_ANCHORS",
            "SCRIPT_BINDING_MIRROR_DOC_ANCHORS",
        ],
    );
    assert_contains_all(
        "current inventory evidence",
        CURRENT_INVENTORY_SOURCE,
        &[
            "Runtime13 current source contract inventory:",
            "unlabelled_host_inventories",
            "mirror_anchors",
        ],
    );
    assert!(MIRROR_DOCS_SOURCE.contains("mod current_inventory;"));
    assert!(MIRROR_DOCS_SOURCE.contains("current_inventory::assert_current_inventory("));
    assert_contains_all(
        "gameplay host child",
        GAMEPLAY_HOST_SOURCE,
        &[
            "runtime_13_gameplay_host_owner_split_keeps_domain_files",
            "GAMEPLAY_HOST_MODULE_ANCHORS",
            "GAMEPLAY_HOST_REGISTRATION_ANCHORS",
        ],
    );
    assert_contains_all(
        "support child",
        SUPPORT_SOURCE,
        &[
            "pub(super) fn assert_files_exist",
            "pub(super) fn assert_file_line_budget",
            "pub(super) fn count_occurrences",
        ],
    );
}

fn assert_line_budget() {
    for (label, source, max_lines) in [
        ("parent route owner", PARENT_SOURCE, 18),
        ("inventory child", INVENTORY_SOURCE, 130),
        ("mirror docs child", MIRROR_DOCS_SOURCE, 140),
        ("current inventory evidence", CURRENT_INVENTORY_SOURCE, 140),
        ("gameplay host child", GAMEPLAY_HOST_SOURCE, 80),
        ("support child", SUPPORT_SOURCE, 50),
        ("split layout guard", SPLIT_LAYOUT_SOURCE, 230),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count <= max_lines,
            "{label} has {line_count} lines; expected at most {max_lines}"
        );
    }
}

fn assert_docs_and_status_mirror_split() {
    for (label, source) in [
        ("Runtime 13 plan", runtime_13_plan()),
        ("Runtime 15 plan", runtime_15_plan()),
        ("runtime index", runtime_index()),
        ("structure convention plan", structure_convention_plan()),
        ("review findings plan", review_findings_plan()),
        ("module convention doc", MODULE_CONVENTION_DOC),
        ("Frameworks 02 plan", frameworks_02_plan()),
    ] {
        assert!(
            source.contains(
                "runtime_15_script_binding_route_owner_split_static_passed_cargo_deferred"
            ),
            "{label} should mirror the script_binding route-owner split status"
        );
    }
    assert_contains_all(
        "module convention doc",
        MODULE_CONVENTION_DOC,
        &[
            "script_binding/inventory.rs",
            "script_binding/mirror_docs.rs",
            "script_binding/gameplay_host.rs",
            "script_binding/support.rs",
            "script_binding/split_layout.rs",
            "runtime_15_script_binding_route_owner_is_folder_backed",
        ],
    );
    assert_contains_all(
        "Frameworks 02 plan",
        frameworks_02_plan(),
        &[
            "frameworks_02_m3_script_binding_route_owner_split_static_passed_cargo_deferred",
            "Runtime 15 M3 script-binding route-owner split",
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
