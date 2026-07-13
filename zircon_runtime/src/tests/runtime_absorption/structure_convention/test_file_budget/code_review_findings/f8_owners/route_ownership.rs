use super::*;

#[path = "route_ownership/child_ownership.rs"]
mod child_ownership;
#[path = "route_ownership/descriptor_builder_routes.rs"]
mod descriptor_builder_routes;
#[path = "route_ownership/descriptor_privacy_routes.rs"]
mod descriptor_privacy_routes;
#[path = "route_ownership/leaf_owners.rs"]
mod leaf_owners;
#[path = "route_ownership/parent_routes.rs"]
mod parent_routes;
#[path = "route_ownership/status_mirrors.rs"]
mod status_mirrors;

pub(super) const F8_ROUTE_PARENT_ROUTES_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_owners/route_ownership/parent_routes.rs";
pub(super) const F8_ROUTE_DESCRIPTOR_BUILDER_ROUTES_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_owners/route_ownership/descriptor_builder_routes.rs";
pub(super) const F8_ROUTE_DESCRIPTOR_PRIVACY_ROUTES_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_owners/route_ownership/descriptor_privacy_routes.rs";
pub(super) const F8_ROUTE_LEAF_OWNERS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_owners/route_ownership/leaf_owners.rs";
pub(super) const F8_ROUTE_CHILD_OWNERSHIP_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_owners/route_ownership/child_ownership.rs";
pub(super) const F8_ROUTE_STATUS_MIRRORS_CHILD: &str = "tests/runtime_absorption/structure_convention/test_file_budget/code_review_findings/f8_owners/route_ownership/status_mirrors.rs";

pub(super) const F8_ROUTE_OWNERSHIP_CHILD_SPLIT_SLICE: &str =
    "Runtime 15 M3 F8 route ownership guard child split";
pub(super) const F8_ROUTE_OWNERSHIP_CHILD_SPLIT_STATUS: &str =
    "runtime_15_f8_route_ownership_guard_child_split_static_passed_cargo_deferred";
pub(super) const F8_ROUTE_OWNERSHIP_CHILD_SPLIT_DATE: &str = "2026-07-05";
pub(super) const F8_ROUTE_OWNERSHIP_CHILD_SPLIT_GUARD: &str =
    "runtime_15_f8_route_ownership_guard_is_child_backed";
pub(super) const F8_ROUTE_STATUS_MIRROR_GUARD: &str =
    "runtime_15_f8_route_ownership_status_mirrors_are_current";

pub(super) const F8_ROUTE_OWNERSHIP_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "parent_routes",
        F8_ROUTE_PARENT_ROUTES_CHILD,
        "assert_f8_parent_routes_are_child_owned",
    ),
    (
        "descriptor_builder_routes",
        F8_ROUTE_DESCRIPTOR_BUILDER_ROUTES_CHILD,
        "assert_f8_descriptor_builder_routes_are_child_owned",
    ),
    (
        "descriptor_privacy_routes",
        F8_ROUTE_DESCRIPTOR_PRIVACY_ROUTES_CHILD,
        "assert_f8_descriptor_privacy_routes_are_child_owned",
    ),
    (
        "leaf_owners",
        F8_ROUTE_LEAF_OWNERS_CHILD,
        "assert_f8_review_leaf_owners_are_child_owned",
    ),
    (
        "child_ownership",
        F8_ROUTE_CHILD_OWNERSHIP_CHILD,
        F8_ROUTE_OWNERSHIP_CHILD_SPLIT_GUARD,
    ),
    (
        "status_mirrors",
        F8_ROUTE_STATUS_MIRRORS_CHILD,
        F8_ROUTE_STATUS_MIRROR_GUARD,
    ),
];

pub(super) fn f8_route_ownership_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, child_path, _) in F8_ROUTE_OWNERSHIP_CHILDREN {
        blob.push_str(&read_runtime_src(child_path));
        blob.push('\n');
    }
    blob
}

#[test]
fn runtime_15_f8_api_convergence_review_guards_are_child_owners() {
    let sources = read_f8_review_sources();

    parent_routes::assert_f8_parent_routes_are_child_owned(&sources);
    descriptor_builder_routes::assert_f8_descriptor_builder_routes_are_child_owned(&sources);
    descriptor_privacy_routes::assert_f8_descriptor_privacy_routes_are_child_owned(&sources);
    leaf_owners::assert_f8_review_leaf_owners_are_child_owned(&sources);
}

#[test]
fn runtime_15_f8_route_ownership_guard_is_child_backed() {
    child_ownership::assert_f8_route_ownership_guard_is_child_backed();
}

#[test]
fn runtime_15_f8_route_ownership_status_mirrors_are_current() {
    status_mirrors::assert_f8_route_ownership_status_mirrors_are_current();
}
