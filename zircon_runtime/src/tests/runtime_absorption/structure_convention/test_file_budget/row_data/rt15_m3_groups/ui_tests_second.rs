use super::*;

const STATUS_ROW_DATA_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/status_row_data_support_maps.rs";
const STATUS_ROW_DATA_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/status_row_data_support_maps.rs";

const UI_TESTS_SECOND_CHILD_ROWS: &[(&str, &str, &str)] = &[
    (
        "component_boundary_rows",
        UI_TESTS_SECOND_COMPONENT_BOUNDARY_ROWS_PATH,
        "Runtime 15 M3 UI component catalog test folder split",
    ),
    (
        "asset_style_rows",
        UI_TESTS_SECOND_ASSET_STYLE_ROWS_PATH,
        "Runtime 15 M3 UI asset test folder split",
    ),
    (
        "runtime_input_rows",
        UI_TESTS_SECOND_RUNTIME_INPUT_ROWS_PATH,
        "Runtime 15 M3 UI runtime input ownership test folder split",
    ),
    (
        "row_data_owner",
        UI_TESTS_SECOND_ROW_DATA_OWNER_ROWS_PATH,
        UI_TESTS_SECOND_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
    ),
];

#[test]
fn runtime_15_ui_tests_second_row_data_is_child_backed() {
    let ui_route = read_runtime_src(UI_TESTS_SECOND_ROW_DATA_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let status_map = read_runtime_src(STATUS_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_ROW_DATA_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "UI tests second route mounts child row groups",
        &ui_route,
        &[
            "ui_tests_second/component_boundary_rows.rs",
            "ui_tests_second/asset_style_rows.rs",
            "ui_tests_second/runtime_input_rows.rs",
            "ui_tests_second/row_data_owner.rs",
            "component_boundary_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "asset_style_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_input_rows::EXPECTED_STATUS_OUTPUT_SLICES",
            "row_data_owner::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M3 UI component catalog test folder split",
        "Runtime 15 M3 UI asset test folder split",
        "Runtime 15 M3 UI runtime input ownership test folder split",
    ] {
        assert!(
            !ui_route.contains(moved_row),
            "ui_tests_second.rs should delegate {moved_row} to child row files"
        );
    }
    for (module_name, path, representative_row) in UI_TESTS_SECOND_CHILD_ROWS {
        let child_source = read_runtime_src(path);
        assert_contains_all(
            "UI tests second child row file keeps representative row",
            &child_source,
            &[*representative_row],
        );
        assert!(
            ui_route.contains(&format!("mod {module_name};")),
            "ui_tests_second.rs should mount {module_name}"
        );
        assert!(
            child_source.lines().count() < 130,
            "{path} should stay below its focused row-data budget"
        );
    }

    assert_contains_all(
        "Runtime 15 M3 aggregation exports UI tests second children",
        &runtime_15_m3,
        &[
            "UI_TESTS_SECOND_ASSET_STYLE_EXPECTED_STATUS_OUTPUT_SLICES",
            "UI_TESTS_SECOND_RUNTIME_INPUT_EXPECTED_STATUS_OUTPUT_SLICES",
            "UI_TESTS_SECOND_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 and top-level aggregation consume UI tests second children",
        &[runtime_15.as_str(), top_level.as_str()].join("\n"),
        &[
            "RUNTIME_15_M3_UI_TESTS_SECOND_ASSET_STYLE_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_UI_TESTS_SECOND_RUNTIME_INPUT_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_15_M3_UI_TESTS_SECOND_ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "M3 status map records UI tests second row-data child split",
        &status_map,
        &[
            UI_TESTS_SECOND_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            UI_TESTS_SECOND_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records UI tests second row-data child split",
        &date_map,
        &[
            UI_TESTS_SECOND_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            "2026-07-07",
        ],
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                UI_TESTS_SECOND_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
                UI_TESTS_SECOND_ROW_DATA_CHILD_SPLIT_STATUS_ID,
                UI_TESTS_SECOND_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
            ],
        );
    }
}
