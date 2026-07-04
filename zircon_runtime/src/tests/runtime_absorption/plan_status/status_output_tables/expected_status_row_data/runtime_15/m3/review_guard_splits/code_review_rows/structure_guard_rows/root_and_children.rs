use super::Slice;

#[path = "root_and_children/code_review_findings.rs"]
mod code_review_findings;
#[path = "root_and_children/f8_child_owner.rs"]
mod f8_child_owner;
#[path = "root_and_children/late_api_cleanup.rs"]
mod late_api_cleanup;
#[path = "root_and_children/p0_native_fixture.rs"]
mod p0_native_fixture;
#[path = "root_and_children/p0_robustness.rs"]
mod p0_robustness;
#[path = "root_and_children/plugin_importer_dx.rs"]
mod plugin_importer_dx;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 code review findings structure guard child-owner split",
        code_review_findings::STRUCTURE_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 code review findings structure guard children folder-backed split",
        code_review_findings::STRUCTURE_GUARD_CHILDREN_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 code review findings structure guard children budget-status child split",
        code_review_findings::STRUCTURE_GUARD_CHILDREN_BUDGET_STATUS_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 code review findings structure guard children root inventory child split",
        code_review_findings::STRUCTURE_GUARD_CHILDREN_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 structure guard plugin-importer child split",
        code_review_findings::STRUCTURE_GUARD_PLUGIN_IMPORTER_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 P0 robustness structure guard folder-backed split",
        p0_robustness::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 P0 robustness root inventory child split",
        p0_robustness::P0_ROBUSTNESS_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 P0 route ownership guard child split",
        p0_robustness::P0_ROUTE_OWNERSHIP_GUARD_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX structure guard folder-backed split",
        plugin_importer_dx::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX structure guard root inventory child split",
        plugin_importer_dx::STRUCTURE_GUARD_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 P0 native fixture leaf-owner guard folder-backed split",
        p0_native_fixture::LEAF_OWNER_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 P0 native fixture leaf-owner root inventory child split",
        p0_native_fixture::P0_NATIVE_FIXTURE_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 F8 child-owner structure guard folder-backed split",
        f8_child_owner::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 F8 child-owner root inventory child split",
        f8_child_owner::F8_CHILD_OWNER_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 F8 route ownership guard child split",
        f8_child_owner::F8_ROUTE_OWNERSHIP_GUARD_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 late API cleanup structure guard folder-backed split",
        late_api_cleanup::STRUCTURE_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 late API cleanup root inventory child split",
        late_api_cleanup::LATE_API_CLEANUP_ROOT_INVENTORY_CHILD_SPLIT,
    ),
];
