use super::*;

#[test]
fn runtime_15_review_guard_moved_row_code_review_rows_are_child_owned() {
    let parent = read_runtime_src(CODE_REVIEW_ROWS_GUARD_PATH);
    for (module_name, path, guard_name) in CODE_REVIEW_ROW_CHILDREN {
        let module_mount = format!("#[path = \"code_review_rows/{module_name}.rs\"]");
        let module_decl = format!("mod {module_name};");
        assert_contains_all(
            "review-guard moved-row code-review rows route mounts child",
            &parent,
            &[module_mount.as_str(), module_decl.as_str()],
        );
        let child = read_runtime_src(path);
        assert_contains_all(path, &child, &[*guard_name]);
    }

    source_delegation::assert_moved_code_review_row_sources_are_delegated();
    review_guard_rows::assert_moved_review_guard_rows_are_child_owned();
    structure_guard_rows::assert_moved_structure_guard_rows_are_child_owned();
    typed_error_structure_rows::assert_moved_typed_error_structure_rows_are_child_owned();
    plugin_importer_rows::assert_moved_plugin_importer_rows_are_child_owned();
}

#[test]
fn runtime_15_review_guard_moved_row_code_review_rows_route_metadata_is_child_owned() {
    let parent = read_runtime_src(CODE_REVIEW_ROWS_GUARD_PATH);
    for (module_name, path) in CODE_REVIEW_ROWS_ROUTE_METADATA_CHILDREN {
        let module_mount = format!("#[path = \"code_review_rows/{module_name}.rs\"]");
        let module_decl = format!("mod {module_name};");
        assert_contains_all(
            "review-guard moved-row code-review rows route metadata child mount",
            &parent,
            &[module_mount.as_str(), module_decl.as_str()],
        );

        let child = read_runtime_src(path);
        let line_count = child.lines().count();
        assert!(
            line_count < 140,
            "{path} should stay focused after code-review rows route metadata split; got {line_count} lines"
        );
    }

    for forbidden in [
        "const CODE_REVIEW_ROWS_GUARD_PATH",
        "const CODE_REVIEW_ROW_CHILDREN",
        "fn code_review_rows_child_source_blob",
        "#[test]",
    ] {
        assert!(
            !parent.contains(forbidden),
            "code_review_rows.rs should delegate route metadata/test owner `{forbidden}`"
        );
    }
    assert!(
        parent.lines().count() < 40,
        "code_review_rows.rs should stay a thin route owner after route metadata split"
    );

    let status_rows = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let status_anchors = [
        CODE_REVIEW_ROWS_ROUTE_METADATA_STATUS_NAME,
        CODE_REVIEW_ROWS_ROUTE_METADATA_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/children.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/child_ownership.rs",
        CODE_REVIEW_ROWS_ROUTE_METADATA_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "production guard review rows record moved-row code-review rows route metadata split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "M3 status-support map records moved-row code-review rows route metadata split",
        &status_map,
        &[
            CODE_REVIEW_ROWS_ROUTE_METADATA_STATUS_NAME,
            CODE_REVIEW_ROWS_ROUTE_METADATA_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records moved-row code-review rows route metadata split",
        &date_map,
        &[CODE_REVIEW_ROWS_ROUTE_METADATA_STATUS_NAME, "2026-07-06"],
    );

    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
}
