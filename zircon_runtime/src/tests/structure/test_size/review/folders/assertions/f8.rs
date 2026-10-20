//! 约束审查守卫汇总的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

#[path = "../f8/budgets.rs"]
mod budgets;
#[path = "../f8/delegation.rs"]
mod delegation;
#[path = "../f8/parent_mounts.rs"]
mod parent_mounts;
#[path = "../../../../../test_size/review/folders/f8/review_children.rs"]
mod review_children;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/child_ownership.rs";
pub(super) const F8_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8.rs";
pub(super) const F8_DIRECT_ASSERTIONS_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/delegation.rs";
pub(super) const F8_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/parent_mounts.rs";
pub(super) const F8_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/review_children.rs";
pub(super) const F8_DIRECT_ASSERTIONS_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/f8/budgets.rs";
pub(super) const F8_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings F8 direct assertions guard folder-backed split";
pub(super) const F8_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS: &str = "runtime_15_code_review_findings_f8_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const F8_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const F8_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_f8_direct_assertions_guard_is_folder_backed";
pub(super) const F8_DIRECT_ASSERTIONS_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_f8_direct_assertions_children_line_budgets_are_current";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) const F8_DIRECT_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        F8_DIRECT_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_f8_direct_assertions_are_child_owner",
    ),
    (
        "parent_mounts",
        F8_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
        F8_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "review_children",
        F8_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD,
        "assert_f8_review_children_are_folder_backed",
    ),
    (
        "budgets",
        F8_DIRECT_ASSERTIONS_BUDGETS_CHILD,
        F8_DIRECT_ASSERTIONS_BUDGET_GUARD,
    ),
];

pub(super) fn assert_f8_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    parent_mounts::assert_f8_api_convergence_parent_mounts_child_owners(sources);
    review_children::assert_f8_review_children_are_folder_backed(sources);
}

pub(super) fn f8_direct_assertion_child_sources() -> Vec<(&'static str, String)> {
    F8_DIRECT_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn f8_direct_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in f8_direct_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn assert_guard_graph_part() {
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/contracts/structure/test_size/review/folders.rs",
        &[
            (
                "child_ownership",
                "#[path = \"../../../../structure/test_size/review/folders/child_ownership.rs\"]",
            ),
            (
                "delegation",
                "#[path = \"../../../../structure/test_size/review/folders/delegation.rs\"]",
            ),
            (
                "direct_review_assertions",
                "#[path = \"../../../../structure/test_size/review/folders/assertions.rs\"]",
            ),
            (
                "source_inventory",
                "#[path = \"../../../../structure/test_size/review/folders/source_inventory.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/f12/review_guard.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/folders/f8/review_children.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/folders/p0/review_children.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/folders/assertions/root_parent.rs",
        &[
            ("backflow", "#[path = \"../root_parent/backflow.rs\"]"),
            ("budgets", "#[path = \"../root_parent/budgets.rs\"]"),
            ("delegation", "#[path = \"../root_parent/delegation.rs\"]"),
            (
                "parent_mounts",
                "#[path = \"../root_parent/parent_mounts.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/source_inventory.rs",
        &[
            ("budgets", "#[path = \"sources/budgets.rs\"]"),
            (
                "delegation",
                "#[path = \"../../../../test_size/review/folders/sources/delegation.rs\"]",
            ),
            ("model", "#[path = \"sources/model.rs\"]"),
            ("reads", "#[path = \"sources/reads.rs\"]"),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/sources/reads.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/late_api_cleanup_owners/root_paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/plugin_importer_dx_child_owners.rs",
        &[
            (
                "budgets",
                "#[path = \"../../structure/test_size/review/importers/budgets.rs\"]",
            ),
            (
                "child_ownership",
                "#[path = \"../../structure/test_size/review/importers/child_ownership.rs\"]",
            ),
            (
                "delegation",
                "#[path = \"../../structure/test_size/review/importers/delegation.rs\"]",
            ),
            (
                "root_child_rows",
                "#[path = \"../../structure/test_size/review/importers/root_child_rows.rs\"]",
            ),
            (
                "root_inventory",
                "#[path = \"../../structure/test_size/review/importers/root_inventory.rs\"]",
            ),
            (
                "root_paths",
                "#[path = \"../../structure/test_size/review/importers/root_paths.rs\"]",
            ),
            (
                "root_sources",
                "#[path = \"../../structure/test_size/review/importers/root_sources.rs\"]",
            ),
            (
                "source_inventory",
                "#[path = \"importers/source_inventory.rs\"]",
            ),
            (
                "structure_assertions",
                "#[path = \"importers/structure_assertions.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/importers/sources/paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/importers/structure/d13_sdk.rs",
        &[
            ("budgets", "#[path = \"d13_sdk/budgets.rs\"]"),
            (
                "parent_mounts",
                "#[path = \"../../../../review/importers/structure/d13_sdk/parent_mounts.rs\"]",
            ),
            ("paths", "#[path = \"d13_sdk/paths.rs\"]"),
            (
                "review_children",
                "#[path = \"../../../../review/importers/structure/d13_sdk/review_children.rs\"]",
            ),
            ("sources", "#[path = \"d13_sdk/sources.rs\"]"),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/importers/structure/d13_sdk/paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/structure/review_mounts/parent_mounts.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/typed_error_child_owners.rs",
        &[
            (
                "budgets",
                "#[path = \"../../../contracts/structure/test_size/review/errors/budgets.rs\"]",
            ),
            ("child_ownership", "#[path = \"errors/child_ownership.rs\"]"),
            ("delegation", "#[path = \"errors/delegation.rs\"]"),
            ("root_child_rows", "#[path = \"errors/root_child_rows.rs\"]"),
            ("root_inventory", "#[path = \"errors/root_inventory.rs\"]"),
            ("root_paths", "#[path = \"errors/root_paths.rs\"]"),
            ("root_sources", "#[path = \"errors/root_sources.rs\"]"),
            (
                "source_inventory",
                "#[path = \"errors/source_inventory.rs\"]",
            ),
            (
                "structure_assertions",
                "#[path = \"../../../test_size/review/errors/structure_assertions.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/errors/sources/paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/errors/structure/convergence_mounts.rs",
        &[
            (
                "asset_parents",
                "#[path = \"../../../../review/errors/convergence_mounts/asset_parents.rs\"]",
            ),
            (
                "budgets",
                "#[path = \"../../../../review/errors/structure/convergence_mounts/budgets.rs\"]",
            ),
            (
                "root_child_rows",
                "#[path = \"../../../../review/errors/convergence_mounts/root_child_rows.rs\"]",
            ),
            (
                "root_inventory",
                "#[path = \"../../../../review/errors/convergence_mounts/root_inventory.rs\"]",
            ),
            (
                "root_paths",
                "#[path = \"../../../../review/errors/convergence_mounts/root_paths.rs\"]",
            ),
            (
                "root_sources",
                "#[path = \"../../../../review/errors/convergence_mounts/root_sources.rs\"]",
            ),
            (
                "runtime_parents",
                "#[path = \"../../../../review/errors/convergence_mounts/runtime_parents.rs\"]",
            ),
            (
                "top_level",
                "#[path = \"../../../../review/errors/structure/convergence_mounts/top_level.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/convergence_mounts/root_child_rows.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/convergence_mounts/runtime_parents.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/moved_guard_absence/parent_backflow.rs",
        &[
            (
                "child_inventory",
                "#[path = \"../parent_backflow/child_inventory.rs\"]",
            ),
            (
                "child_ownership",
                "#[path = \"../parent_backflow/child_ownership.rs\"]",
            ),
            (
                "guard_body",
                "#[path = \"../structure/parent_backflow/guard_body.rs\"]",
            ),
            (
                "guard_names",
                "#[path = \"../structure/parent_backflow/guard_names.rs\"]",
            ),
            (
                "metadata",
                "#[path = \"../structure/parent_backflow/metadata.rs\"]",
            ),
            (
                "parent_paths",
                "#[path = \"../structure/parent_backflow/parent_paths.rs\"]",
            ),
            (
                "sources",
                "#[path = \"../structure/parent_backflow/sources.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/parent_backflow/parent_paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/moved_guard_absence/root_paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/errors/structure/loader/routes.rs",
        &[
            ("abi_surfaces", "#[path = \"../../../../../review/errors/structure/loader/routes/abi_surfaces.rs\"]"),
            ("child_inventory", "#[path = \"../../../../../review/errors/loader/routes/child_inventory.rs\"]"),
            ("child_ownership", "#[path = \"../../../../../review/errors/loader/routes/child_ownership.rs\"]"),
            ("lifecycle_paths", "#[path = \"../../../../../review/errors/loader/routes/lifecycle_paths.rs\"]"),
            ("live_host", "#[path = \"../../../../../review/errors/structure/loader/routes/live_host.rs\"]"),
            ("manifest_sources", "#[path = \"../../../../../review/errors/loader/routes/manifest_sources.rs\"]"),
            ("metadata", "#[path = \"../../../../../review/errors/structure/loader/routes/metadata.rs\"]"),
            ("plugin_descriptor", "#[path = \"../../../../../review/errors/loader/routes/plugin_descriptor.rs\"]"),
            ("replay_runtime", "#[path = \"../../../../../review/errors/structure/loader/routes/replay_runtime.rs\"]"),
            ("source_helper_ownership", "#[path = \"../../../../../review/errors/loader/routes/source_helper_ownership.rs\"]"),
            ("sources", "#[path = \"../../../../../review/errors/structure/loader/routes/sources.rs\"]"),
            ("top_level", "#[path = \"../../../../../review/errors/structure/loader/routes/top_level.rs\"]"),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/loader/routes/live_host.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/loader/routes/replay_runtime.rs",
        &[],
    );
}
