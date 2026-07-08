use super::*;

#[test]
fn runtime_15_foundation_guards_row_data_owner_is_child_backed() {
    let foundation_guards = read_runtime_src(FOUNDATION_GUARDS_ROWS_PATH);
    let dead_code_surface = read_runtime_src(FOUNDATION_GUARDS_DEAD_CODE_SURFACE_PATH);
    let runtime_structure_tests = read_runtime_src(FOUNDATION_GUARDS_RUNTIME_STRUCTURE_TESTS_PATH);
    let runtime_structure_test_children = [
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_CORE_RUNTIME_ROWS_PATH,
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_ROOT_ROUTE_ROWS_PATH,
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_RUNTIME_ABSORPTION_CORE_ROWS_PATH,
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_RUNTIME_ABSORPTION_PLATFORM_ROWS_PATH,
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_TEST_GUARD_ROWS_PATH,
        FOUNDATION_GUARDS_RUNTIME_STRUCTURE_ROW_DATA_OWNER_PATH,
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n");
    let plugin_importer_review = read_runtime_src(FOUNDATION_GUARDS_PLUGIN_IMPORTER_REVIEW_PATH);
    let plugin_importer_migrations =
        read_runtime_src(FOUNDATION_GUARDS_PLUGIN_IMPORTER_MIGRATIONS_PATH);
    let runtime_absorption_followups =
        read_runtime_src(FOUNDATION_GUARDS_RUNTIME_ABSORPTION_FOLLOWUPS_PATH);
    let row_data_owner = read_runtime_src(FOUNDATION_GUARDS_ROW_DATA_OWNER_PATH);
    let row_children = [
        dead_code_surface.as_str(),
        runtime_structure_tests.as_str(),
        runtime_structure_test_children.as_str(),
        plugin_importer_review.as_str(),
        plugin_importer_migrations.as_str(),
        runtime_absorption_followups.as_str(),
        row_data_owner.as_str(),
    ]
    .join("\n");

    assert_contains_all(
        "Runtime 15 foundation-guards row-data parent mounts child owners",
        &foundation_guards,
        &[
            "#[path = \"foundation_guards/dead_code_surface.rs\"]",
            "#[path = \"foundation_guards/runtime_structure_tests.rs\"]",
            "#[path = \"foundation_guards/plugin_importer_review.rs\"]",
            "#[path = \"foundation_guards/plugin_importer_migrations.rs\"]",
            "#[path = \"foundation_guards/runtime_absorption_followups.rs\"]",
            "#[path = \"foundation_guards/row_data_owner.rs\"]",
            "dead_code_surface::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_structure_tests::EXPECTED_STATUS_OUTPUT_SLICES",
            "plugin_importer_review::EXPECTED_STATUS_OUTPUT_SLICES",
            "plugin_importer_migrations::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_absorption_followups::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert!(
        !foundation_guards.contains(
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &["
        ),
        "foundation_guards.rs should route child row-data owners instead of owning row tuples directly"
    );
    assert_contains_all(
        "Runtime 15 foundation-guards row-data children own representative rows",
        &row_children,
        &[
            "Runtime 15 M3 graphics dead-code guard module split",
            "Runtime 15 M3 runtime dead-code production-gate status wording cleanup",
            "Runtime 15 M3 root entries guard child-owner split",
            "Runtime 15 M3 D8 runtime registration builder original evidence paths",
            "Runtime 15 M3 D13 importer manifest parity guard",
            "Runtime 15 M3 input manager test folder split",
            CHILD_OWNER_STATUS_NAME,
            CHILD_OWNER_STATUS_ID,
            CHILD_OWNER_GUARD_NAME,
        ],
    );
}
