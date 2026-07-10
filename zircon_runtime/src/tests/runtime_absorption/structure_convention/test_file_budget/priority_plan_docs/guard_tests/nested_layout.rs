use super::*;

const CHILD_PROSE_SLICE: &str =
    "Runtime 15 M3 priority plan docs guard-test child prose full inventory sync";
const CHILD_PROSE_STATUS: &str =
    "runtime_15_priority_plan_docs_guard_test_child_prose_full_inventory_sync_static_passed_cargo_deferred";
const CHILD_PROSE_GUARD_PATH: &str =
    "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory";
const NESTED_CHILD_PATHS: &[&str] = &[
    "structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs",
    "structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs",
    "structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs",
    "structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs",
    "structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs",
];

#[test]
fn runtime_15_priority_plan_docs_guard_test_children_are_folder_backed() {
    let parent_path =
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs";
    let parent = read_runtime_src(parent_path);
    let child_specs = [
        (
            "child_layout",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs",
            "runtime_15_priority_plan_docs_guard_children_are_folder_backed",
        ),
        (
            "listing",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs",
            "runtime_15_priority_plan_docs_guard_tests_stay_listed",
        ),
        (
            "inventory_sync",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs",
            "runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources",
        ),
        (
            "nested_layout",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs",
            "runtime_15_priority_plan_docs_guard_test_children_are_folder_backed",
        ),
        (
            "moved_paths",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs",
            "runtime_15_priority_plan_docs_moved_guard_paths_stay_current",
        ),
    ];

    assert!(
        !parent.contains("#[test]"),
        "priority_plan_docs/guard_tests.rs should route nested children and keep test bodies in child owners"
    );
    assert!(
        parent.lines().count() <= 40,
        "priority_plan_docs/guard_tests.rs should stay a small nested route owner"
    );

    for (module_name, child_path, guard_name) in child_specs {
        let module_mount = format!("mod {module_name};");
        assert!(
            parent.contains(&module_mount),
            "priority_plan_docs/guard_tests.rs should mount nested child `{module_mount}`"
        );

        let child = read_runtime_src(child_path);
        assert_contains_all(child_path, &child, &["use super::*;", guard_name]);
        assert!(
            child.lines().count() <= 220,
            "priority plan docs guard-test child owner `{child_path}` should stay focused and below 220 lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = priority_plan_doc_owner_row_source();
    let status_map = priority_plan_doc_status_map_source();
    let date_map = priority_plan_doc_date_map_source();

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 priority plan docs guard-test child-owner split",
                "runtime_15_priority_plan_docs_guard_test_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs",
                "structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs",
                "structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs",
                "structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs",
                "structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs",
                "structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs",
                "runtime_15_priority_plan_docs_guard_test_children_are_folder_backed",
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[
            "Runtime 15 M3 priority plan docs guard-test child-owner split",
            "runtime_15_priority_plan_docs_guard_test_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[
            "Runtime 15 M3 priority plan docs guard-test child-owner split",
            "2026-07-01",
        ],
    );
}

#[test]
fn runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = priority_plan_doc_owner_row_source();
    let status_map = priority_plan_doc_status_map_source();
    let date_map = priority_plan_doc_date_map_source();

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                CHILD_PROSE_SLICE,
                CHILD_PROSE_STATUS,
                CHILD_PROSE_GUARD_PATH,
                "full priority-plan-doc guard-test child inventory",
                "Cargo gate deferred",
            ],
        );
    }

    let current_owner_archive = priority_plan_doc_current_owner_archive_source();
    assert_child_owner_windows_name_full_inventory(
        "priority-plan-doc current-owner archive",
        &current_owner_archive,
    );

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[CHILD_PROSE_SLICE, CHILD_PROSE_STATUS],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[CHILD_PROSE_SLICE, "2026-07-04"],
    );
}

fn assert_child_owner_windows_name_full_inventory(label: &str, source: &str) {
    let lines: Vec<&str> = source.lines().collect();
    let indexes: Vec<usize> = lines.iter().enumerate().filter_map(|(index, line)| line.contains("runtime_15_priority_plan_docs_guard_test_child_owner_split_static_passed_cargo_deferred").then_some(index)).collect();
    assert!(
        !indexes.is_empty(),
        "{label} should keep a guard-test child-owner split status anchor"
    );

    for index in indexes {
        let window = lines[index..usize::min(index + 32, lines.len())].join("\n");
        assert_contains_all(
            label,
            &window,
            &["full priority-plan-doc guard-test child inventory"],
        );
        for child_path in NESTED_CHILD_PATHS {
            assert!(
                window.contains(child_path),
                "{label} guard-test child-owner status should mention `{child_path}`"
            );
        }
    }
}
