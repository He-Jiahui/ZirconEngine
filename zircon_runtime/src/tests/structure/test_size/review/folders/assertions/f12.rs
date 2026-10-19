//! 约束审查守卫汇总的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

#[path = "../f12/budgets.rs"]
mod budgets;
#[path = "../f12/delegation.rs"]
mod delegation;
#[path = "../f12/review_guard.rs"]
mod review_guard;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs";
pub(super) const F12_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12.rs";
pub(super) const F12_DIRECT_ASSERTIONS_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/delegation.rs";
pub(super) const F12_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/review_guard.rs";
pub(super) const F12_DIRECT_ASSERTIONS_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f12/budgets.rs";
pub(super) const F12_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings F12 direct assertions guard folder-backed split";
pub(super) const F12_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS: &str = "runtime_15_code_review_findings_f12_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const F12_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const F12_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_f12_direct_assertions_guard_is_folder_backed";
pub(super) const F12_DIRECT_ASSERTIONS_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_f12_direct_assertions_children_line_budgets_are_current";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) const F12_DIRECT_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        F12_DIRECT_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_f12_direct_assertions_are_child_owner",
    ),
    (
        "review_guard",
        F12_DIRECT_ASSERTIONS_REVIEW_GUARD_CHILD,
        F12_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "budgets",
        F12_DIRECT_ASSERTIONS_BUDGETS_CHILD,
        F12_DIRECT_ASSERTIONS_BUDGET_GUARD,
    ),
];

pub(super) fn assert_f12_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    review_guard::assert_f12_dead_code_review_guard_is_child_owned(sources);
}

pub(super) fn f12_direct_assertion_child_sources() -> Vec<(&'static str, String)> {
    F12_DIRECT_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn f12_direct_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in f12_direct_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn assert_guard_graph_part() {
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/contracts/structure/test_size/review.rs",
        &[
            ("f8_child_owners", "#[path = \"review/f8_child_owners.rs\"]"),
            ("folder_backed_summary", "#[path = \"review/folders.rs\"]"),
            (
                "late_api_cleanup_child_owners",
                "#[path = \"../../../test_size/review/late_api_cleanup_child_owners.rs\"]",
            ),
            ("p0_child_owners", "#[path = \"review/p0_child_owners.rs\"]"),
            (
                "p0_native_fixture_leaf_owners",
                "#[path = \"review/fixtures.rs\"]",
            ),
            (
                "plugin_importer_dx_child_owners",
                "#[path = \"../../../test_size/review/plugin_importer_dx_child_owners.rs\"]",
            ),
            (
                "structure_guard_children",
                "#[path = \"review/children.rs\"]",
            ),
            (
                "typed_error_child_owners",
                "#[path = \"../../../structure/test_size/review/typed_error_child_owners.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/assertions/f12.rs",
        &[
            ("budgets", "#[path = \"../f12/budgets.rs\"]"),
            ("delegation", "#[path = \"../f12/delegation.rs\"]"),
            ("review_guard", "#[path = \"../f12/review_guard.rs\"]"),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/f8/parent_mounts.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/p0/parent_mounts.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/folders/render/review_guard.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/folders/root_parent/parent_mounts.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/sources/model.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/late_api_cleanup_owners/root_child_rows.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/late_api_cleanup_owners/route_ownership.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/importers/sources/budgets.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/importers/structure_assertions.rs",
        &[
            (
                "child_ownership",
                "#[path = \"structure/child_ownership.rs\"]",
            ),
            ("d13_sdk", "#[path = \"structure/d13_sdk.rs\"]"),
            ("delegation", "#[path = \"structure/delegation.rs\"]"),
            ("review_mounts", "#[path = \"structure/review_mounts.rs\"]"),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/structure/d13_sdk/parent_mounts/review_mounts.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/importers/structure/review_mounts.rs",
        &[
            (
                "budgets",
                "#[path = \"../../../../review/importers/structure/review_mounts/budgets.rs\"]",
            ),
            (
                "parent_mounts",
                "#[path = \"../../../../review/structure/review_mounts/parent_mounts.rs\"]",
            ),
            (
                "paths",
                "#[path = \"../../../../review/importers/structure/review_mounts/paths.rs\"]",
            ),
            (
                "review_children",
                "#[path = \"../../../../review/structure/review_mounts/review_children.rs\"]",
            ),
            (
                "sources",
                "#[path = \"../../../../review/importers/structure/review_mounts/sources.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/structure/review_mounts/review_children.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/errors/sources/budgets.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/errors/structure_assertions.rs",
        &[
            (
                "child_ownership",
                "#[path = \"structure/child_ownership.rs\"]",
            ),
            (
                "convergence_mounts",
                "#[path = \"structure/convergence_mounts.rs\"]",
            ),
            ("delegation", "#[path = \"structure/delegation.rs\"]"),
            (
                "moved_guard_absence",
                "#[path = \"../../../review/errors/structure/moved_guard_absence.rs\"]",
            ),
            (
                "native_plugin_loader",
                "#[path = \"../../../structure/test_size/review/errors/structure/loader.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/convergence_mounts/budgets.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/convergence_mounts/root_sources.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/moved_guard_absence.rs",
        &[
            ("budgets", "#[path = \"moved_guard_absence/budgets.rs\"]"),
            (
                "child_ownership",
                "#[path = \"../moved_guard_absence/child_ownership.rs\"]",
            ),
            (
                "parent_backflow",
                "#[path = \"../moved_guard_absence/parent_backflow.rs\"]",
            ),
            (
                "path_anchors",
                "#[path = \"../moved_guard_absence/path_anchors.rs\"]",
            ),
            (
                "preserved_guards",
                "#[path = \"../moved_guard_absence/preserved_guards.rs\"]",
            ),
            (
                "root_child_rows",
                "#[path = \"../moved_guard_absence/root_child_rows.rs\"]",
            ),
            (
                "root_inventory",
                "#[path = \"../moved_guard_absence/root_inventory.rs\"]",
            ),
            (
                "root_paths",
                "#[path = \"../moved_guard_absence/root_paths.rs\"]",
            ),
            (
                "root_sources",
                "#[path = \"../moved_guard_absence/root_sources.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/parent_backflow/guard_names.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/moved_guard_absence/preserved_guards.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/errors/structure/loader/metadata.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/loader/routes/lifecycle_paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/loader/routes/plugin_descriptor.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/errors/structure/loader/sources.rs",
        &[],
    );
}
