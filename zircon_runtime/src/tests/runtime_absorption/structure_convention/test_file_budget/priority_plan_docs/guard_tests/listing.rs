use super::*;

#[test]
fn runtime_15_priority_plan_docs_guard_tests_stay_listed() {
    for (label, path) in PRIORITY_PLAN_DOCS {
        let source = read_repo(path);
        assert_priority_plan_doc_tests_list_guard_functions(label, path, &source);
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = runtime_index_with_output_archive_source();
    let structure_plan = read_repo("docs/plans/engine-code-structure-convention.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = priority_plan_doc_owner_row_source();
    let status_map = priority_plan_doc_status_map_source();
    let date_map = priority_plan_doc_date_map_source();

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("structure convention plan", structure_plan.as_str()),
        ("review findings plan", review_findings.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 priority plan docs guard-test listing guard",
                "runtime_15_priority_plan_docs_guard_test_listing_guard_static_passed_cargo_deferred",
                "docs/plans/engine-code-structure-convention.md",
                "docs/plans/engine-code-review-findings-2026-06.md",
                "structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs",
                "runtime_15_priority_plan_docs_guard_tests_stay_listed",
                "priority_plan_docs/code_paths.rs::runtime_15_priority_plan_docs_code_paths_stay_current",
                "priority_plan_docs/test_paths.rs::runtime_15_priority_plan_docs_test_paths_stay_current",
                "priority_plan_docs/frontmatter_status.rs::runtime_15_priority_plan_docs_frontmatter_status_stays_current",
                "priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
                "priority_plan_docs/header_sections.rs::runtime_15_priority_plan_docs_required_header_sections_stay_complete",
                "priority_plan_docs/plan_sources.rs::runtime_15_priority_plan_docs_plan_sources_stay_cross_linked",
                "priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed",
                "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources",
                "priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory",
                "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_children_are_folder_backed",
                "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory",
                "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_guard_paths_stay_current",
                "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory",
                "Cargo gate deferred",
            ],
        );
    }

    assert_contains_all(
        "status expected-slice map",
        &status_map,
        &[
            "Runtime 15 M3 priority plan docs guard-test listing guard",
            "runtime_15_priority_plan_docs_guard_test_listing_guard_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date expected-slice map",
        &date_map,
        &[
            "Runtime 15 M3 priority plan docs guard-test listing guard",
            "2026-07-01",
        ],
    );
}

fn assert_priority_plan_doc_tests_list_guard_functions(label: &str, path: &str, source: &str) {
    let frontmatter = frontmatter_lines(label, path, source);
    let test_entries = frontmatter_section_items(&frontmatter, "tests");

    for guard in PRIORITY_PLAN_DOC_GUARDS {
        let expected_prefix = "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/";
        assert!(
            test_entries
                .iter()
                .any(|entry| entry.starts_with(expected_prefix) && entry.contains(guard)),
            "{label} priority plan doc `{path}` should list priority-plan-doc guard `{guard}` in frontmatter tests"
        );
    }
}
