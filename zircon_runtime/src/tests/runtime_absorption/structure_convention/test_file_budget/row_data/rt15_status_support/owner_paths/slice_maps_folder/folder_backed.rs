use super::*;

const EXPECTED_SLICE_OWNER_PATH_GUARD_CHILDREN: &[(&str, &str)] = &[
    (
        "budget_traversal",
        "assert_expected_slice_owner_path_budget_traversal_is_current",
    ),
    (
        "route_children",
        "assert_expected_slice_owner_path_route_exposes_child_groups",
    ),
    (
        "status_current",
        "assert_expected_slice_owner_paths_status_is_current",
    ),
];

#[test]
fn runtime_15_status_support_expected_slice_owner_paths_guard_is_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps_folder_backed.rs",
    );
    let child_blob = expected_slice_owner_path_guard_child_source_blob();
    for (module_name, representative_fn) in EXPECTED_SLICE_OWNER_PATH_GUARD_CHILDREN {
        let module_mount =
            format!("#[path = \"slice_maps_folder/{module_name}.rs\"]");
        assert_contains_all(
            "expected-slice owner paths guard mounts child modules",
            &parent,
            &[module_mount.as_str(), &format!("mod {module_name};")],
        );
        assert_contains_all(
            "expected-slice owner paths guard child owns moved assertions",
            &child_blob,
            &[representative_fn],
        );
    }
    assert_contains_all(
        "expected-slice owner paths guard parent routes original test",
        &parent,
        &[
            "runtime_15_status_support_expected_slice_owner_paths_are_folder_backed",
            "route_children::assert_expected_slice_owner_path_route_exposes_child_groups",
            "budget_traversal::assert_expected_slice_owner_path_budget_traversal_is_current",
            "status_current::assert_expected_slice_owner_paths_status_is_current",
        ],
    );
    assert!(
        !parent.contains("const EXPECTED_SLICE_OWNER_PATH_CHILDREN"),
        "expected_slice_maps_folder_backed.rs should route guard checks instead of owning child inventory"
    );
    assert!(
        !parent.contains("let status_anchors = ["),
        "expected_slice_maps_folder_backed.rs should route status-current assertions to child modules"
    );
    assert_expected_slice_owner_paths_guard_status_is_current();
}

fn expected_slice_owner_path_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (module_name, _) in EXPECTED_SLICE_OWNER_PATH_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(&format!(
            "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/slice_maps_folder/{module_name}.rs"
        )));
        blob.push('\n');
    }
    blob.push_str(&route_children::expected_slice_owner_path_child_source_blob());
    blob
}

fn assert_expected_slice_owner_paths_guard_status_is_current() {
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_SUPPORT_PRIORITY_ROWS_PATH);
    assert_contains_all(
        "expected-slice owner paths guard status row",
        &status_rows,
        &[
            EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_STATUS_NAME,
            EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_STATUS_ID,
            "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps_folder_backed.rs",
            "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/slice_maps_folder/route_children.rs",
            "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/slice_maps_folder/budget_traversal.rs",
            "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/slice_maps_folder/status_current.rs",
            EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );

    let doc_anchors = [
        EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_STATUS_NAME,
        EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps_folder_backed.rs",
        EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for path in [
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        "docs/plans/zircon_runtime/runtime/index.md",
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
        "docs/plans/engine-code-review-findings-2026-06.md",
        "docs/plans/engine-code-structure-convention.md",
        "docs/zircon_runtime/structure/module-convention.md",
        ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
    ] {
        assert_contains_all(path, &read_repo(path), &doc_anchors);
    }
    assert_contains_all(
        "M3 status-support status map records expected-slice owner paths guard split",
        &read_runtime_src(STATUS_SUPPORT_ROW_DATA_STATUS_MAP_PATH),
        &[
            EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_STATUS_NAME,
            EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records expected-slice owner paths guard split",
        &read_runtime_src(STATUS_SUPPORT_ROW_DATA_DATE_MAP_PATH),
        &[
            EXPECTED_SLICE_OWNER_PATHS_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-07",
        ],
    );
}
