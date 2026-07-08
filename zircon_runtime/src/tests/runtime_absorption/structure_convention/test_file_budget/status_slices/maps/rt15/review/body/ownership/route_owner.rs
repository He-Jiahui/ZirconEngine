use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_guard_body_is_child_owned() {
    let parent = read_runtime_src(STRUCTURE_REVIEW_GUARD_BODY);
    let children = read_review_root_sources(STRUCTURE_REVIEW_GUARD_BODY_CHILDREN);
    let child_ownership_children = CHILD_OWNERSHIP_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "review guard root guard-body route",
        &parent,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"body/guard_status.rs\"]",
            "mod guard_status;",
            "#[path = \"body/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );

    for moved_anchor in [
        "#[test]",
        "runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned",
        "Runtime 15 M3 structure-support expected-slice map child-owner split",
        "Cargo gate deferred active Render Plan08 lane",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "review/guard_body.rs should delegate moved guard body {moved_anchor}"
        );
    }

    assert_contains_all(
        "review guard root guard-body children",
        &format!("{children}\n{child_ownership_children}"),
        &[
            "runtime_15_review_guard_expected_slice_structure_guard_tests_are_child_owned",
            "runtime_15_review_guard_expected_slice_root_children_own_moved_checks",
            "runtime_15_review_guard_expected_slice_structure_guard_body_sources_stay_budgeted",
            "runtime_15_review_guard_expected_slice_root_guard_body_status_is_mirrored",
            ROOT_GUARD_GUARD,
            CHILD_OWNERSHIP_GUARD,
        ],
    );
}
