use super::*;

pub(super) fn assert_priority_plan_doc_guard_row_data_sources_are_child_owned() {
    let priority_plan_docs = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs.rs",
    );
    let listing_guard = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs",
    );
    let moved_paths_guard = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs",
    );

    assert_contains_all(
        "priority-plan-doc guard inventory",
        &priority_plan_docs,
        &[
            "runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries",
            GUARD,
        ],
    );
    assert_contains_all(
        "priority-plan-doc guard listing",
        &listing_guard,
        &[
            FRONTMATTER_UNIQUENESS_GUARD_PATH,
            INVENTORY_SYNC_GUARD_PATH,
            LISTING_PROSE_GUARD_PATH,
            "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory",
            "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory",
        ],
    );
    assert_contains_all(
        "priority-plan-doc moved guard paths",
        &moved_paths_guard,
        &[
            FRONTMATTER_UNIQUENESS_GUARD_PATH,
            INVENTORY_SYNC_GUARD_PATH,
            LISTING_PROSE_GUARD_PATH,
            "priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory",
            "priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory",
        ],
    );
}
