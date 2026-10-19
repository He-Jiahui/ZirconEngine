//! 约束审查守卫汇总的模块挂载、委托入口与既有检查保留；读取当前源码后按文本验证，不能替代被检查模块的行为测试。
use super::super::super::super::*;

use super::super::source_inventory::CodeReviewFindingsSources;

#[path = "../p0/budgets.rs"]
mod budgets;
#[path = "../p0/delegation.rs"]
mod delegation;
#[path = "../p0/parent_mounts.rs"]
mod parent_mounts;
#[path = "../../../../../test_size/review/folders/p0/review_children.rs"]
mod review_children;

pub(super) const DIRECT_REVIEW_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions.rs";
pub(super) const P0_DIRECT_ASSERTIONS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0.rs";
pub(super) const P0_DIRECT_ASSERTIONS_DELEGATION_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/delegation.rs";
pub(super) const P0_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/parent_mounts.rs";
pub(super) const P0_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/review_children.rs";
pub(super) const P0_DIRECT_ASSERTIONS_BUDGETS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/folder_backed_summary/direct_review_assertions/p0/budgets.rs";
pub(super) const P0_DIRECT_ASSERTIONS_FOLDER_BACKED_SLICE: &str =
    "Runtime 15 M3 code review findings P0 direct assertions guard folder-backed split";
pub(super) const P0_DIRECT_ASSERTIONS_FOLDER_BACKED_STATUS: &str = "runtime_15_code_review_findings_p0_direct_assertions_guard_folder_backed_static_passed_cargo_deferred";
pub(super) const P0_DIRECT_ASSERTIONS_FOLDER_BACKED_DATE: &str = "2026-07-04";
pub(super) const P0_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD: &str =
    "runtime_15_code_review_findings_p0_direct_assertions_guard_is_folder_backed";
pub(super) const P0_DIRECT_ASSERTIONS_BUDGET_GUARD: &str =
    "runtime_15_code_review_findings_p0_direct_assertions_children_line_budgets_are_current";
pub(super) const CODE_REVIEW_FINDINGS_LINE_BUDGET: usize = 800;

pub(super) const P0_DIRECT_ASSERTIONS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "delegation",
        P0_DIRECT_ASSERTIONS_DELEGATION_CHILD,
        "runtime_15_code_review_findings_p0_direct_assertions_are_child_owner",
    ),
    (
        "parent_mounts",
        P0_DIRECT_ASSERTIONS_PARENT_MOUNTS_CHILD,
        P0_DIRECT_ASSERTIONS_FOLDER_BACKED_GUARD,
    ),
    (
        "review_children",
        P0_DIRECT_ASSERTIONS_REVIEW_CHILDREN_CHILD,
        "assert_p0_review_children_are_folder_backed",
    ),
    (
        "budgets",
        P0_DIRECT_ASSERTIONS_BUDGETS_CHILD,
        P0_DIRECT_ASSERTIONS_BUDGET_GUARD,
    ),
];

pub(super) fn assert_p0_direct_sources_are_folder_backed(sources: &CodeReviewFindingsSources) {
    parent_mounts::assert_p0_robustness_parent_mounts_child_owners(sources);
    review_children::assert_p0_review_children_are_folder_backed(sources);
}

pub(super) fn p0_direct_assertion_child_sources() -> Vec<(&'static str, String)> {
    P0_DIRECT_ASSERTIONS_GUARD_CHILDREN
        .iter()
        .map(|(_, path, _)| (*path, read_runtime_src(path)))
        .collect()
}

pub(super) fn p0_direct_assertion_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, source) in p0_direct_assertion_child_sources() {
        blob.push_str(&source);
        blob.push('\n');
    }
    blob
}

pub(super) fn assert_guard_graph_part() {
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/assertions.rs",
        &[
            (
                "child_ownership",
                "#[path = \"../../../../test_size/review/folders/assertions/child_ownership.rs\"]",
            ),
            (
                "delegation",
                "#[path = \"../../../../test_size/review/folders/assertions/delegation.rs\"]",
            ),
            ("f12", "#[path = \"assertions/f12.rs\"]"),
            ("f8", "#[path = \"assertions/f8.rs\"]"),
            ("p0", "#[path = \"assertions/p0.rs\"]"),
            ("render", "#[path = \"assertions/render.rs\"]"),
            (
                "root_parent",
                "#[path = \"../../../../test_size/review/folders/assertions/root_parent.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/assertions/f8.rs",
        &[
            ("budgets", "#[path = \"../f8/budgets.rs\"]"),
            ("delegation", "#[path = \"../f8/delegation.rs\"]"),
            ("parent_mounts", "#[path = \"../f8/parent_mounts.rs\"]"),
            (
                "review_children",
                "#[path = \"../../../../../test_size/review/folders/f8/review_children.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/assertions/p0.rs",
        &[
            ("budgets", "#[path = \"../p0/budgets.rs\"]"),
            ("delegation", "#[path = \"../p0/delegation.rs\"]"),
            ("parent_mounts", "#[path = \"../p0/parent_mounts.rs\"]"),
            (
                "review_children",
                "#[path = \"../../../../../test_size/review/folders/p0/review_children.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/assertions/render.rs",
        &[
            ("budgets", "#[path = \"../render/budgets.rs\"]"),
            ("delegation", "#[path = \"../render/delegation.rs\"]"),
            (
                "review_guard",
                "#[path = \"../../../../../test_size/review/folders/render/review_guard.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/folders/root_parent/backflow.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/folders/sources/budgets.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/late_api_cleanup_child_owners.rs",
        &[
            (
                "budgets",
                "#[path = \"late_api_cleanup_owners/budgets.rs\"]",
            ),
            (
                "delegation",
                "#[path = \"late_api_cleanup_owners/delegation.rs\"]",
            ),
            (
                "root_child_rows",
                "#[path = \"../../review/late_api_cleanup_owners/root_child_rows.rs\"]",
            ),
            (
                "root_inventory",
                "#[path = \"../../review/late_api_cleanup_owners/root_inventory.rs\"]",
            ),
            (
                "root_paths",
                "#[path = \"late_api_cleanup_owners/root_paths.rs\"]",
            ),
            (
                "root_sources",
                "#[path = \"../../review/late_api_cleanup_owners/root_sources.rs\"]",
            ),
            (
                "route_ownership",
                "#[path = \"../../review/late_api_cleanup_owners/route_ownership.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/late_api_cleanup_owners/root_sources.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/test_size/review/importers/source_inventory.rs",
        &[
            (
                "budgets",
                "#[path = \"../../../structure/test_size/review/importers/sources/budgets.rs\"]",
            ),
            ("delegation", "#[path = \"sources/delegation.rs\"]"),
            (
                "paths",
                "#[path = \"../../../structure/test_size/review/importers/sources/paths.rs\"]",
            ),
            (
                "reads",
                "#[path = \"../../../structure/test_size/review/importers/sources/reads.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/importers/sources/reads.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/importers/structure/d13_sdk/parent_mounts.rs",
        &[
            (
                "child_ownership",
                "#[path = \"../../../d13_sdk/parent_mounts/child_ownership.rs\"]",
            ),
            (
                "delegation",
                "#[path = \"../../../structure/d13_sdk/parent_mounts/delegation.rs\"]",
            ),
            (
                "folder_backed",
                "#[path = \"../../../structure/d13_sdk/parent_mounts/folder_backed.rs\"]",
            ),
            (
                "review_mounts",
                "#[path = \"../../../structure/d13_sdk/parent_mounts/review_mounts.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/importers/structure/d13_sdk/review_children.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/importers/structure/review_mounts/paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/errors/source_inventory.rs",
        &[
            ("budgets", "#[path = \"sources/budgets.rs\"]"),
            (
                "child_inventory",
                "#[path = \"../../../../test_size/review/errors/sources/child_inventory.rs\"]",
            ),
            (
                "child_sources",
                "#[path = \"../../../../test_size/review/errors/sources/child_sources.rs\"]",
            ),
            ("delegation", "#[path = \"sources/delegation.rs\"]"),
            ("metadata", "#[path = \"sources/metadata.rs\"]"),
            ("paths", "#[path = \"sources/paths.rs\"]"),
            ("reads", "#[path = \"sources/reads.rs\"]"),
            (
                "source_helper_ownership",
                "#[path = \"../../../../review/errors/sources/source_helper_ownership.rs\"]",
            ),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/errors/sources/reads.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/convergence_mounts/asset_parents.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/convergence_mounts/root_paths.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/convergence_mounts/top_level.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/parent_backflow/guard_body.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/parent_backflow/sources.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/structure/test_size/review/errors/structure/loader.rs",
        &[
            ("budgets", "#[path = \"../../../../../test_size/review/errors/structure/loader/budgets.rs\"]"),
            ("child_inventory", "#[path = \"../../../../../review/errors/structure/loader/child_inventory.rs\"]"),
            ("delegation", "#[path = \"../../../../../test_size/review/errors/structure/loader/delegation.rs\"]"),
            ("metadata", "#[path = \"../../../../../test_size/review/errors/structure/loader/metadata.rs\"]"),
            ("routes", "#[path = \"../../../../../test_size/review/errors/structure/loader/routes.rs\"]"),
            ("source_helper_ownership", "#[path = \"../../../../../review/errors/loader/source_helper_ownership.rs\"]"),
            ("sources", "#[path = \"../../../../../test_size/review/errors/structure/loader/sources.rs\"]"),
        ],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/loader/routes/abi_surfaces.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/loader/routes/manifest_sources.rs",
        &[],
    );
    super::super::source_inventory::assert_guard_owner_contract(
        "tests/review/errors/structure/loader/routes/top_level.rs",
        &[],
    );
}
