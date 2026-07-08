use super::*;

#[test]
fn runtime_15_test_file_budget_root_layout_assertions_guard_is_folder_backed() {
    let assertions = read_runtime_src(ASSERTIONS_OWNER);
    let assertions_asset_children = read_runtime_src(ASSERTIONS_ASSET_CHILDREN_OWNER);
    let assertions_parent_mounts = read_runtime_src(ASSERTIONS_PARENT_MOUNTS_OWNER);
    let assertions_render_status_children =
        read_runtime_src(ASSERTIONS_RENDER_STATUS_CHILDREN_OWNER);
    let assertions_runtime_scene_children =
        read_runtime_src(ASSERTIONS_RUNTIME_SCENE_CHILDREN_OWNER);
    let assertions_ui_children = read_runtime_src(ASSERTIONS_UI_CHILDREN_OWNER);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let framework_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs",
    );
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/root_layout_ui_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/root_layout_ui_maps.rs",
    );
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "assertions route owns only child mounts and dispatch",
        &assertions,
        &[
            "#[path = \"assertions/asset_children.rs\"]",
            "mod asset_children;",
            "#[path = \"assertions/parent_mounts.rs\"]",
            "mod parent_mounts;",
            "#[path = \"assertions/render_status_children.rs\"]",
            "mod render_status_children;",
            "#[path = \"assertions/runtime_scene_children.rs\"]",
            "mod runtime_scene_children;",
            "#[path = \"assertions/ui_children.rs\"]",
            "mod ui_children;",
            "parent_mounts::assert_parent_mounts_and_moved_guards",
            "asset_children::assert_asset_children",
            "runtime_scene_children::assert_runtime_scene_children",
            "render_status_children::assert_render_status_children",
            "ui_children::assert_ui_children",
        ],
    );
    for moved_anchor in [
        "asset test-budget child owns child-owner mounts",
        "dynamic-scene absorption test-budget child owns absorption guard",
        "scene ECS systems test-budget child owns ECS systems guard",
        "status output expected-slices test-budget parent mounts child guard owners",
        "UI shared core test-budget parent mounts shared-core child owners",
    ] {
        assert!(
            !assertions.contains(moved_anchor),
            "assertions.rs should stay route-only instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "assertions parent-mount child owns root parent checks",
        &assertions_parent_mounts,
        &[
            "assert_parent_mounts_and_moved_guards",
            "test file budget parent mounts folder-backed guard owners",
            "test_file_budget/mod.rs should mount child guard owners",
        ],
    );
    assert_contains_all(
        "assertions asset child owns asset checks",
        &assertions_asset_children,
        &[
            "assert_asset_children",
            "asset test-budget child owns child-owner mounts",
            "asset project test-budget child owns project guards",
            "asset glTF primitive fixture test-budget child owns fixture guard",
        ],
    );
    assert_contains_all(
        "assertions runtime-scene child owns runtime and scene checks",
        &assertions_runtime_scene_children,
        &[
            "assert_runtime_scene_children",
            "code review findings test-budget child owns findings guard",
            "core runtime deactivation test-budget child owns deactivation guard",
            "scene ECS reflect foundation test-budget child owns foundation guard",
        ],
    );
    assert_contains_all(
        "assertions render-status child owns render, script, ECS, and status checks",
        &assertions_render_status_children,
        &[
            "assert_render_status_children",
            "RHI device-contract test-budget child owns device-contract guard",
            "script VM test-budget child owns script VM guards",
            "status output row-data Runtime 15 child owns Runtime 15 row-data guard",
        ],
    );
    assert_contains_all(
        "assertions UI child owns UI checks",
        &assertions_ui_children,
        &[
            "assert_ui_children",
            "UI shared core test-budget parent mounts shared-core child owners",
            "UI v2 asset test-budget child owns historical v2-asset guard",
        ],
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
            assertions.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/asset_children.rs",
            assertions_asset_children.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/parent_mounts.rs",
            assertions_parent_mounts.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/render_status_children.rs",
            assertions_render_status_children.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/runtime_scene_children.rs",
            assertions_runtime_scene_children.as_str(),
        ),
        (
            "structure_convention/test_file_budget/root_layout/folder_backed/assertions/ui_children.rs",
            assertions_ui_children.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 220,
            "{path} should stay below the nested Runtime 15 assertion-owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status row data", status_rows.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 test file budget root-layout assertions guard folder-backed split",
                "runtime_15_test_file_budget_root_layout_assertions_guard_folder_backed_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/root_layout/folder_backed/assertions.rs",
                "structure_convention/test_file_budget/root_layout/folder_backed/assertions/parent_mounts.rs",
                "structure_convention/test_file_budget/root_layout/folder_backed/assertions/render_status_children.rs",
                "runtime_15_test_file_budget_root_layout_assertions_guard_is_folder_backed",
            ],
        );
    }
    for (label, source) in [
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02 plan", framework_plan.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 test file budget root-layout assertions guard folder-backed split",
                "runtime_15_test_file_budget_root_layout_assertions_guard_folder_backed_static_passed_cargo_deferred",
                "runtime_15_test_file_budget_root_layout_assertions_guard_is_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status map",
        &status_map,
        &[
            "Runtime 15 M3 test file budget root-layout assertions guard folder-backed split",
            "runtime_15_test_file_budget_root_layout_assertions_guard_folder_backed_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date map",
        &date_map,
        &[
            "Runtime 15 M3 test file budget root-layout assertions guard folder-backed split",
            "2026-07-05",
        ],
    );
}
