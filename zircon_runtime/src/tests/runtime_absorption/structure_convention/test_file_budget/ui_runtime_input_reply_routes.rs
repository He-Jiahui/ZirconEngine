use super::*;

#[path = "ui_runtime_input_reply_routes/root.rs"]
mod root;
#[path = "ui_runtime_input_reply_routes/route_children.rs"]
mod route_children;
#[path = "ui_runtime_input_reply_routes/table_pointer.rs"]
mod table_pointer;

const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");
const ROOT_GUARD: &str = concat!(
    "fn runtime_15_ui_runtime_input_reply_routes_",
    "tests_are_folder_backed"
);
const ROUTE_CHILDREN_GUARD: &str = concat!(
    "fn runtime_15_ui_runtime_input_reply_route_",
    "children_are_folder_backed"
);
const TABLE_POINTER_GUARD: &str = concat!(
    "fn runtime_15_ui_runtime_input_reply_table_pointer_",
    "routes_are_folder_backed"
);

#[test]
fn runtime_15_ui_runtime_input_reply_route_guard_child_owners_are_folder_backed() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs",
    );
    let root = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes/root.rs",
    );
    let route_children = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes/route_children.rs",
    );
    let table_pointer = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes/table_pointer.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_first.rs",
    );
    let status_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/ui_maps.rs",
        ),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/ui_maps.rs",
        ),
    ]
    .join("\n");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "UI runtime input reply route guard parent mounts child owners",
        &parent,
        &[
            "mod root;",
            "mod route_children;",
            "mod table_pointer;",
            "runtime_15_ui_runtime_input_reply_route_guard_child_owners_are_folder_backed",
        ],
    );
    for moved_guard in [ROOT_GUARD, ROUTE_CHILDREN_GUARD, TABLE_POINTER_GUARD] {
        assert!(
            !parent.contains(moved_guard),
            "structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs should mount child owners instead of defining {moved_guard}"
        );
    }
    assert_contains_all(
        "UI runtime input reply route child guards preserve existing ownership checks",
        &format!("{root}\n{route_children}\n{table_pointer}"),
        &[ROOT_GUARD, ROUTE_CHILDREN_GUARD, TABLE_POINTER_GUARD],
    );

    let test_count = [
        parent.as_str(),
        root.as_str(),
        route_children.as_str(),
        table_pointer.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        test_count, 4,
        "UI runtime input reply route guard parent plus children should preserve three existing guards plus the new layout guard"
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/ui_runtime_input_reply_routes/root.rs",
            root.as_str(),
        ),
        (
            "structure_convention/test_file_budget/ui_runtime_input_reply_routes/route_children.rs",
            route_children.as_str(),
        ),
        (
            "structure_convention/test_file_budget/ui_runtime_input_reply_routes/table_pointer.rs",
            table_pointer.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 focused guard budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "UI runtime input reply route guard status map",
        &status_map,
        &[
            "Runtime 15 M3 UI runtime input reply route guard child-owner split",
            "runtime_15_ui_runtime_input_reply_route_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "UI runtime input reply route guard date map",
        &date_map,
        &[
            "Runtime 15 M3 UI runtime input reply route guard child-owner split",
            "2026-06-25",
        ],
    );
    for (label, source) in [
        ("status-output M3 UI row data", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI runtime input reply route guard child-owner split",
                "runtime_15_ui_runtime_input_reply_route_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs",
                "structure_convention/test_file_budget/ui_runtime_input_reply_routes/route_children.rs",
                "runtime_15_ui_runtime_input_reply_route_guard_child_owners_are_folder_backed",
            ],
        );
    }
}
