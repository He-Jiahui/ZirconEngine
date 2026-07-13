use super::*;

#[test]
fn runtime_15_status_output_expected_slice_legacy_guard_body_is_folder_backed() {
    let legacy_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/legacy_maps.rs",
    );
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{LEGACY_GUARD_BODY_PARENT}"
    ));
    let children = LEGACY_GUARD_BODY_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "legacy maps parent keeps guard-body route",
        &legacy_parent,
        &["#[path = \"legacy_maps/guard_body.rs\"]", "mod guard_body;"],
    );
    assert_contains_all(
        "legacy guard-body parent mounts focused children",
        &parent,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"body/legacy_routes.rs\"]",
            "mod legacy_routes;",
            "#[path = \"body/paths.rs\"]",
            "mod paths;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
        ],
    );

    for moved_anchor in [
        "#[test]",
        "let status_parent = read_runtime_src(",
        "legacy status/date children own pre-Runtime-15 branches",
        "Runtime 14 Cargo 验证窗口探测",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "legacy_maps/guard_body.rs should delegate moved guard-body anchor {moved_anchor}"
        );
    }

    assert_contains_all(
        "legacy guard-body children own moved checks",
        &children,
        &[
            LEGACY_MAPS_GUARD,
            LEGACY_GUARD_BODY_GUARD,
            "runtime_15_status_output_expected_slice_legacy_guard_body_children_stay_budgeted",
            "runtime_15_status_output_expected_slice_legacy_guard_body_status_is_synced",
            "legacy status/date children own pre-Runtime-15 branches",
            "Runtime 14 Cargo 验证窗口探测",
        ],
    );
}
