use super::*;

const CHILD_PROSE_SLICE: &str = "Runtime 15 M3 priority plan docs child prose full inventory sync";
const CHILD_PROSE_STATUS: &str =
    "runtime_15_priority_plan_docs_child_prose_full_inventory_sync_static_passed_cargo_deferred";
const CHILD_PROSE_GUARD_PATH: &str = "priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed";
const PRIORITY_CHILD_PATHS: &[&str] = &[
    "structure_convention/test_file_budget/priority_plan_docs/code_paths.rs",
    "structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs",
    "structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs",
    "structure_convention/test_file_budget/priority_plan_docs/header_sections.rs",
    "structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs",
    "structure_convention/test_file_budget/priority_plan_docs/test_paths.rs",
];

#[test]
fn runtime_15_priority_plan_docs_guard_children_are_folder_backed() {
    let parent_path =
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs.rs";
    let parent = read_runtime_src(parent_path);
    let child_specs = [
        (
            "code_paths",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/code_paths.rs",
            "runtime_15_priority_plan_docs_code_paths_stay_current",
        ),
        (
            "test_paths",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/test_paths.rs",
            "runtime_15_priority_plan_docs_test_paths_stay_current",
        ),
        (
            "frontmatter_uniqueness",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs",
            "runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
        ),
        (
            "header_sections",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/header_sections.rs",
            "runtime_15_priority_plan_docs_required_header_sections_stay_complete",
        ),
        (
            "plan_sources",
            "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs",
            "runtime_15_priority_plan_docs_plan_sources_stay_cross_linked",
        ),
    ];

    assert!(
        !parent.contains("#[test]"),
        "priority_plan_docs.rs should route children and keep test bodies in folder-backed child owners"
    );
    assert!(
        parent.lines().count() <= 120,
        "priority_plan_docs.rs should remain a small route/shared-helper owner"
    );

    for (module_name, child_path, guard_name) in child_specs {
        let module_mount = format!("mod {module_name};");
        assert!(
            parent.contains(&module_mount),
            "priority_plan_docs.rs should mount child module `{module_mount}`"
        );

        let child = read_runtime_src(child_path);
        assert_contains_all(child_path, &child, &["use super::*;", guard_name]);
        assert!(
            child.lines().count() <= 220,
            "priority plan docs child owner `{child_path}` should stay focused and below 220 lines"
        );
    }

    assert!(
        parent.contains("mod guard_tests;"),
        "priority_plan_docs.rs should mount the guard_tests child owner"
    );
    let guard_tests_parent_path = "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests.rs";
    let guard_tests_parent = read_runtime_src(guard_tests_parent_path);
    assert_contains_all(
        guard_tests_parent_path,
        &guard_tests_parent,
        &[
            "use super::*;",
            "mod child_layout;",
            "mod inventory_sync;",
            "mod listing;",
            "mod moved_paths;",
            "mod nested_layout;",
        ],
    );
    assert!(
        !guard_tests_parent.contains("#[test]"),
        "priority_plan_docs/guard_tests.rs should route nested guard-test children only"
    );
    assert!(
        guard_tests_parent.lines().count() <= 40,
        "priority_plan_docs/guard_tests.rs should remain a small nested route owner"
    );

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let current_owner_archive = priority_plan_doc_current_owner_archive_source();
    assert_child_owner_windows_name_full_inventory(
        "priority-plan-doc current-owner archive",
        &current_owner_archive,
    );
}

fn assert_child_owner_windows_name_full_inventory(label: &str, source: &str) {
    let lines: Vec<&str> = source.lines().collect();
    let indexes: Vec<usize> = lines.iter().enumerate().filter_map(|(index, line)| line.contains("runtime_15_priority_plan_docs_guard_child_owner_split_static_passed_cargo_deferred").then_some(index)).collect();
    assert!(
        !indexes.is_empty(),
        "{label} should keep a priority-plan-doc child-owner split status anchor"
    );

    for index in indexes {
        let window = lines[index..usize::min(index + 32, lines.len())].join("\n");
        assert_contains_all(label, &window, &["full priority-plan-doc child inventory"]);
        for child_path in PRIORITY_CHILD_PATHS {
            assert!(
                window.contains(child_path),
                "{label} child-owner status should mention `{child_path}`"
            );
        }
    }
}
