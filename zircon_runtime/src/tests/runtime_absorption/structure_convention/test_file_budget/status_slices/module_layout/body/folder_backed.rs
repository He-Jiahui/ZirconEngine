use super::*;

#[test]
fn runtime_15_expected_slice_module_layout_guard_body_is_folder_backed() {
    let module_layout =
        read_runtime_src(&format!("tests/runtime_absorption/{MODULE_LAYOUT_PARENT}"));
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{MODULE_LAYOUT_GUARD_BODY_PARENT}"
    ));
    let children = MODULE_LAYOUT_GUARD_BODY_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "module-layout parent keeps guard-body route",
        &module_layout,
        &[
            "#[path = \"module_layout/guard_body.rs\"]",
            "mod guard_body;",
        ],
    );
    assert_contains_all(
        "module-layout guard-body parent mounts focused children",
        &parent,
        &[
            "#[path = \"body/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"body/child_ownership.rs\"]",
            "mod child_ownership;",
            "#[path = \"body/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"body/paths.rs\"]",
            "mod paths;",
            "#[path = \"body/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"body/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
        ],
    );

    for moved_anchor in [
        "#[test]",
        "let parent = read_runtime_src(",
        "expected-slice guard parent mounts child guard owners",
        "Runtime 15 status rows record expected-slice guard child-owner split",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "module_layout/guard_body.rs should delegate moved guard-body anchor {moved_anchor}"
        );
    }

    assert_contains_all(
        "module-layout guard-body children own moved checks",
        &children,
        &[
            EXPECTED_SLICE_GUARD,
            MODULE_LAYOUT_GUARD_BODY_GUARD,
            "runtime_15_expected_slice_module_layout_guard_body_children_stay_budgeted",
            "runtime_15_expected_slice_module_layout_guard_body_status_is_synced",
            "expected-slice guard parent mounts child guard owners",
            "Runtime 15 status rows record expected-slice guard child-owner split",
        ],
    );
}
