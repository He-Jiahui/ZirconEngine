use super::*;

#[path = "route_ownership/child_ownership.rs"]
mod child_ownership;
#[path = "route_ownership/leaf_owners.rs"]
mod leaf_owners;
#[path = "route_ownership/parent_routes.rs"]
mod parent_routes;

pub(super) const P0_ROUTE_PARENT_ROUTES_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_owners/route_ownership/parent_routes.rs";
pub(super) const P0_ROUTE_LEAF_OWNERS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_owners/route_ownership/leaf_owners.rs";
pub(super) const P0_ROUTE_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/p0_owners/route_ownership/child_ownership.rs";

pub(super) const P0_ROUTE_OWNERSHIP_CHILD_SPLIT_SLICE: &str =
    "Runtime 15 M3 P0 route ownership guard child split";
pub(super) const P0_ROUTE_OWNERSHIP_CHILD_SPLIT_STATUS: &str =
    "runtime_15_p0_route_ownership_guard_child_split_static_passed_cargo_deferred";
pub(super) const P0_ROUTE_OWNERSHIP_CHILD_SPLIT_DATE: &str = "2026-07-05";
pub(super) const P0_ROUTE_OWNERSHIP_CHILD_SPLIT_GUARD: &str =
    "runtime_15_p0_route_ownership_guard_is_child_backed";

pub(super) const P0_ROUTE_OWNERSHIP_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "parent_routes",
        P0_ROUTE_PARENT_ROUTES_CHILD,
        "assert_p0_parent_routes_are_child_owned",
    ),
    (
        "leaf_owners",
        P0_ROUTE_LEAF_OWNERS_CHILD,
        "assert_p0_review_leaf_owners_are_child_owned",
    ),
    (
        "child_ownership",
        P0_ROUTE_CHILD_OWNERSHIP_CHILD,
        "assert_p0_route_ownership_guard_is_child_backed",
    ),
];

pub(super) fn p0_route_ownership_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, child_path, _) in P0_ROUTE_OWNERSHIP_CHILDREN {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob
}

pub(super) fn assert_p0_robustness_child_owners_are_folder_backed() {
    let sources = read_p0_robustness_sources();

    parent_routes::assert_p0_parent_routes_are_child_owned(&sources);
    leaf_owners::assert_p0_review_leaf_owners_are_child_owned(&sources);
}

#[test]
fn runtime_15_p0_robustness_review_guards_are_child_owners() {
    assert_p0_robustness_child_owners_are_folder_backed();
}

#[test]
fn runtime_15_p0_route_ownership_guard_is_child_backed() {
    child_ownership::assert_p0_route_ownership_guard_is_child_backed();
}
