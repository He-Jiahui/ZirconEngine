use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_root_guard_body_child_ownership_is_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{CHILD_OWNERSHIP_ROUTE_PATH}"
    ));
    let children = CHILD_OWNERSHIP_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "review guard root guard-body child ownership route owner",
        &parent,
        &[
            "#[path = \"ownership/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"ownership/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"ownership/moved_checks.rs\"]",
            "mod moved_checks;",
            "#[path = \"ownership/paths.rs\"]",
            "mod paths;",
            "#[path = \"ownership/route_owner.rs\"]",
            "mod route_owner;",
            "#[path = \"ownership/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "let structure_support_guard_child =",
        "Runtime 15 M3 structure-support expected-slice map child-owner split",
        "Cargo gate deferred active Render Plan08 lane",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "body/child_ownership.rs should delegate moved body {moved_anchor}"
        );
    }
    assert_contains_all(
        "review guard root guard-body child ownership children",
        &children,
        &[
            "runtime_15_review_guard_expected_slice_root_guard_body_is_child_owned",
            "runtime_15_review_guard_expected_slice_root_children_own_moved_checks",
            "runtime_15_review_guard_expected_slice_root_guard_body_child_ownership_sources_stay_budgeted",
            "runtime_15_review_guard_expected_slice_root_guard_body_child_ownership_status_is_mirrored",
            CHILD_OWNERSHIP_SLICE,
            CHILD_OWNERSHIP_STATUS,
            CHILD_OWNERSHIP_GUARD,
        ],
    );
}
